// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{Arg, ArgAction, ArgMatches, Command, crate_version};
use std::fs::{self, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use uucore::error::UResult;
use uzers::{get_group_by_gid, get_user_by_uid};

/// Which effective rights comments to print
#[derive(Debug, Clone, Copy, PartialEq)]
enum Effective {
    /// Only where the mask removes permissions (the default)
    Some,
    /// For every entry the mask applies to
    All,
    /// Never
    None,
}

/// Configuration structure to hold all getfacl options
#[derive(Debug)]
struct Config {
    omit_header: bool,
    show_access: bool,
    show_default: bool,
    effective: Effective,
    absolute_names: bool,
}

impl Config {
    /// Create a Config from command-line arguments
    fn from_matches(matches: &ArgMatches) -> Self {
        let access = matches.get_flag("access");
        let default = matches.get_flag("default");
        // The later of -e and -E wins
        let effective = match (
            matches
                .index_of("all-effective")
                .filter(|_| matches.get_flag("all-effective")),
            matches
                .index_of("no-effective")
                .filter(|_| matches.get_flag("no-effective")),
        ) {
            (Some(all), Some(none)) if all > none => Effective::All,
            (_, Some(_)) => Effective::None,
            (Some(_), None) => Effective::All,
            (None, None) => Effective::Some,
        };
        Config {
            omit_header: matches.get_flag("omit-header"),
            // Both lists unless only one of -a and -d is given
            show_access: access || !default,
            show_default: default || !access,
            effective,
            absolute_names: matches.get_flag("absolute-names"),
        }
    }
}

const ACL_ACCESS_XATTR: &str = "system.posix_acl_access";
const ACL_DEFAULT_XATTR: &str = "system.posix_acl_default";
const ACL_XATTR_VERSION: u32 = 2;

const ACL_USER_OBJ: u16 = 0x01;
const ACL_USER: u16 = 0x02;
const ACL_GROUP_OBJ: u16 = 0x04;
const ACL_GROUP: u16 = 0x08;
const ACL_MASK: u16 = 0x10;
const ACL_OTHER: u16 = 0x20;

/// One entry of a POSIX ACL
#[derive(Debug, Clone, Copy)]
struct AclEntry {
    tag: u16,
    perm: u16,
    id: u32,
}

/// Decodes the extended attribute the kernel stores a POSIX ACL in: a little
/// endian version 2 header, then 8 byte entries of tag, permissions and id.
fn parse_acl_xattr(value: &[u8]) -> Option<Vec<AclEntry>> {
    let (header, entries) = value.split_at_checked(4)?;
    if u32::from_le_bytes(header.try_into().ok()?) != ACL_XATTR_VERSION || entries.len() % 8 != 0 {
        return None;
    }
    Some(
        entries
            .as_chunks::<8>()
            .0
            .iter()
            .map(|e| AclEntry {
                tag: u16::from_le_bytes([e[0], e[1]]),
                perm: u16::from_le_bytes([e[2], e[3]]),
                id: u32::from_le_bytes([e[4], e[5], e[6], e[7]]),
            })
            .collect(),
    )
}

fn read_acl(file: &str, name: &str) -> Option<Vec<AclEntry>> {
    xattr::get_deref(file, name)
        .ok()
        .flatten()
        .and_then(|value| parse_acl_xattr(&value))
}

/// The entries the permission bits of `mode` stand for, for a file without an ACL
fn acl_from_mode(mode: u32) -> Vec<AclEntry> {
    [(ACL_USER_OBJ, 6), (ACL_GROUP_OBJ, 3), (ACL_OTHER, 0)]
        .into_iter()
        .map(|(tag, shift)| AclEntry {
            tag,
            perm: ((mode >> shift) & 0o7) as u16,
            id: u32::MAX,
        })
        .collect()
}

fn perm_string(perm: u16) -> String {
    format!(
        "{}{}{}",
        if perm & 0o4 != 0 { "r" } else { "-" },
        if perm & 0o2 != 0 { "w" } else { "-" },
        if perm & 0o1 != 0 { "x" } else { "-" }
    )
}

fn user_name(uid: u32) -> String {
    get_user_by_uid(uid).map_or_else(
        || uid.to_string(),
        |u| u.name().to_string_lossy().into_owned(),
    )
}

fn group_name(gid: u32) -> String {
    get_group_by_gid(gid).map_or_else(
        || gid.to_string(),
        |g| g.name().to_string_lossy().into_owned(),
    )
}

fn print_acl(entries: &[AclEntry], prefix: &str, effective: Effective) {
    let mask = entries.iter().find(|e| e.tag == ACL_MASK).map(|e| e.perm);
    for entry in entries {
        let (kind, qualifier) = match entry.tag {
            ACL_USER_OBJ => ("user", String::new()),
            ACL_USER => ("user", user_name(entry.id)),
            ACL_GROUP_OBJ => ("group", String::new()),
            ACL_GROUP => ("group", group_name(entry.id)),
            ACL_MASK => ("mask", String::new()),
            ACL_OTHER => ("other", String::new()),
            _ => continue,
        };
        let mut line = format!("{prefix}{kind}:{qualifier}:{}", perm_string(entry.perm));
        // The mask limits named users and all groups
        if let Some(mask) = mask
            && matches!(entry.tag, ACL_USER | ACL_GROUP_OBJ | ACL_GROUP)
        {
            let rights = entry.perm & mask;
            if effective == Effective::All || (effective == Effective::Some && rights != entry.perm)
            {
                line.push_str(&format!("\t#effective:{}", perm_string(rights)));
            }
        }
        println!("{line}");
    }
}

/// Compute the name shown in the `# file:` header.
///
/// Like GNU getfacl, unless `--absolute-names` is given, leading slashes are
/// stripped (with a one-time warning) and a leading `./` is removed. An empty
/// result is shown as `.`.
fn display_name<'a>(file_path: &'a str, config: &Config, warned: &mut bool) -> &'a str {
    if config.absolute_names {
        return file_path;
    }

    let name = if file_path.starts_with('/') {
        if !*warned {
            eprintln!(
                "{}: Removing leading '/' from absolute path names",
                uucore::util_name()
            );
            *warned = true;
        }
        file_path.trim_start_matches('/')
    } else if let Some(rest) = file_path.strip_prefix("./") {
        rest.trim_start_matches('/')
    } else {
        file_path
    };

    if name.is_empty() { "." } else { name }
}

fn print_header(file_path: &str, metadata: &Metadata) -> std::io::Result<()> {
    // Fetching owner and group names
    let owner = get_user_by_uid(metadata.uid())
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());
    let group = get_group_by_gid(metadata.gid())
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    println!("# file: {file_path}");
    println!("# owner: {owner}");
    println!("# group: {group}");

    Ok(())
}

fn print_output(
    file_path: &str,
    name: &str,
    config: &Config,
    metadata: &Metadata,
) -> std::io::Result<()> {
    // Print header unless omitted
    if !config.omit_header {
        print_header(name, metadata)?;
    }

    if config.show_access {
        let acl = read_acl(file_path, ACL_ACCESS_XATTR)
            .unwrap_or_else(|| acl_from_mode(metadata.permissions().mode()));
        print_acl(&acl, "", config.effective);
    }

    if config.show_default
        && metadata.is_dir()
        && let Some(acl) = read_acl(file_path, ACL_DEFAULT_XATTR)
    {
        // Without the access ACL next to them, the entries need no prefix
        let prefix = if config.show_access { "default:" } else { "" };
        print_acl(&acl, prefix, config.effective);
    }

    Ok(())
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Parse configuration from command-line arguments
    let config = Config::from_matches(&matches);
    let mut warned_absolute = false;

    // Handle file arguments
    if let Some(files) = matches.get_many::<String>("file") {
        for file in files {
            let metadata = fs::metadata(file)?;

            let name = display_name(file, &config, &mut warned_absolute);
            print_output(file, name, &config, &metadata)?;
            println!();
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    const USAGE: &str = "getfacl [-aceEsRLPtpndvh] file ...
       getfacl [-aceEsRLPtpndvh] -";

    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Get file access control lists")
        .override_usage(USAGE)
        .arg(
            Arg::new("access")
                .short('a')
                .long("access")
                .action(ArgAction::SetTrue)
                .help("Display the file access control list"),
        )
        .arg(
            Arg::new("default")
                .short('d')
                .long("default")
                .action(ArgAction::SetTrue)
                .help("Display the default access control list"),
        )
        .arg(
            Arg::new("omit-header")
                .short('c')
                .long("omit-header")
                .action(ArgAction::SetTrue)
                .help("Do not display the comment header"),
        )
        .arg(
            Arg::new("all-effective")
                .short('e')
                .long("all-effective")
                .action(ArgAction::SetTrue)
                .help("Print all effective rights comments"),
        )
        .arg(
            Arg::new("no-effective")
                .short('E')
                .long("no-effective")
                .action(ArgAction::SetTrue)
                .help("Do not print effective rights comments"),
        )
        .arg(
            Arg::new("skip-base")
                .short('s')
                .long("skip-base")
                .action(ArgAction::SetTrue)
                .help("Skip files that only have the base ACL entries"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .action(ArgAction::SetTrue)
                .help("List the ACLs of all files and directories recursively"),
        )
        .arg(
            Arg::new("logical")
                .short('L')
                .long("logical")
                .action(ArgAction::SetTrue)
                .help("Logical walk, follow symbolic links to directories"),
        )
        .arg(
            Arg::new("physical")
                .short('P')
                .long("physical")
                .action(ArgAction::SetTrue)
                .help("Physical walk, do not follow symbolic links"),
        )
        .arg(
            Arg::new("tabular")
                .short('t')
                .long("tabular")
                .action(ArgAction::SetTrue)
                .help("Use an alternative tabular output format"),
        )
        .arg(
            Arg::new("absolute-names")
                .short('p')
                .long("absolute-names")
                .action(ArgAction::SetTrue)
                .help("Do not strip leading slash characters"),
        )
        .arg(
            Arg::new("numeric")
                .short('n')
                .long("numeric")
                .action(ArgAction::SetTrue)
                .help("List numeric user and group IDs"),
        )
        .arg(
            Arg::new("file")
                .help("File to get ACLs for")
                .required(false)
                .action(ArgAction::Append)
                .value_name("FILE"),
        )
}
