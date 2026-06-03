// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{Arg, ArgAction, Command, crate_version, ArgMatches};
use std::fs::{self, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use uucore::error::UResult;
use uzers::{get_group_by_gid, get_user_by_uid};

/// Configuration structure to hold all getfacl options
#[derive(Debug)]
struct Config {
    omit_header: bool,
    show_access: bool,
}

impl Config {
    /// Create a Config from command-line arguments
    fn from_matches(matches: &ArgMatches) -> Self {
        Config {
            omit_header: matches.get_flag("omit-header"),
            // Always show access ACL (either by default or when --access flag is set)
            show_access: true,
        }
    }
}

fn print_file_acl(metadata: &Metadata) -> std::io::Result<()> {

    // Fetching and formatting file permissions
    let perms = metadata.permissions();
    let mode = perms.mode();
    let user_perms = format!(
        "{}{}{}",
        if mode & 0o400 != 0 { "r" } else { "-" },
        if mode & 0o200 != 0 { "w" } else { "-" },
        if mode & 0o100 != 0 { "x" } else { "-" }
    );
    let group_perms = format!(
        "{}{}{}",
        if mode & 0o040 != 0 { "r" } else { "-" },
        if mode & 0o020 != 0 { "w" } else { "-" },
        if mode & 0o010 != 0 { "x" } else { "-" }
    );
    let other_perms = format!(
        "{}{}{}",
        if mode & 0o004 != 0 { "r" } else { "-" },
        if mode & 0o002 != 0 { "w" } else { "-" },
        if mode & 0o001 != 0 { "x" } else { "-" }
    );

    println!("user::{user_perms}");
    println!("group::{group_perms}");
    println!("other::{other_perms}");

    Ok(())
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
    config: &Config,
    metadata: &Metadata,
) -> std::io::Result<()> {
    // Print header unless omitted
    if !config.omit_header {
        print_header(file_path, metadata)?;
    }

    // Print access ACL if requested as we always show access ACL
    if config.show_access {
        print_file_acl(metadata)?;
    }

    Ok(())
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Parse configuration from command-line arguments
    let config = Config::from_matches(&matches);

    // Handle file arguments
    if let Some(files) = matches.get_many::<String>("file") {
        for file in files {
            let metadata = fs::metadata(file)?;

            print_output(file, &config, &metadata)?;

            // Implement the logic to fetch and display the ACLs using xattr
            match xattr::list(file) {
                Ok(attrs) => {
                    for attr in attrs.peekable() {
                        match xattr::get(file, &attr) {
                            Ok(Some(value)) => {
                                // Assuming the ACL data is stored in a specific xattr key, you might need to adjust this
                                println!("Attribute: {:?}, Value: {:?}", attr.to_str(), value);
                            }
                            Ok(None) => println!("Attribute {:?} has no value.", attr.to_str()),
                            Err(e) => {
                                println!("Error getting attribute {:?}: {}", attr.to_str(), e)
                            }
                        }
                    }
                }
                Err(e) => println!("Error listing attributes for file {file}: {e}"),
            }
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
