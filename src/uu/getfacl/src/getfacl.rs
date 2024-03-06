// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, ArgAction, Command};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use uucore::{error::UResult, help_about, help_usage};
use uzers::{get_group_by_gid, get_user_by_uid};

const ABOUT: &str = help_about!("getfacl.md");
const USAGE: &str = help_usage!("getfacl.md");

fn print_file_acl(file_path: &str) -> std::io::Result<()> {
    let metadata = fs::metadata(file_path)?;

    // Fetching owner and group names
    let owner = get_user_by_uid(metadata.uid())
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());
    let group = get_group_by_gid(metadata.gid())
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

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

    // Generating the output
    println!("# file: {}", file_path);
    println!("# owner: {}", owner);
    println!("# group: {}", group);
    println!("user::{}", user_perms);
    println!("group::{}", group_perms);
    println!("other::{}", other_perms);

    Ok(())
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Example: Handle file arguments
    if let Some(files) = matches.get_many::<String>("file") {
        for file in files {
            print_file_acl(file)?;

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
                Err(e) => println!("Error listing attributes for file {}: {}", file, e),
            }
        }
    }

    Ok(())
}
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(USAGE)
        .arg(
            Arg::new("access")
                .short('a')
                .long("access")
                .help("Display the file access control list"),
        )
        .arg(
            Arg::new("default")
                .short('d')
                .long("default")
                .help("Display the default access control list"),
        )
        .arg(
            Arg::new("omit-header")
                .short('c')
                .long("omit-header")
                .help("Do not display the comment header"),
        )
        .arg(
            Arg::new("all-effective")
                .short('e')
                .long("all-effective")
                .help("Print all effective rights comments"),
        )
        .arg(
            Arg::new("no-effective")
                .short('E')
                .long("no-effective")
                .help("Do not print effective rights comments"),
        )
        .arg(
            Arg::new("skip-base")
                .short('s')
                .long("skip-base")
                .help("Skip files that only have the base ACL entries"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .help("List the ACLs of all files and directories recursively"),
        )
        .arg(
            Arg::new("logical")
                .short('L')
                .long("logical")
                .help("Logical walk, follow symbolic links to directories"),
        )
        .arg(
            Arg::new("physical")
                .short('P')
                .long("physical")
                .help("Physical walk, do not follow symbolic links"),
        )
        .arg(
            Arg::new("tabular")
                .short('t')
                .long("tabular")
                .help("Use an alternative tabular output format"),
        )
        .arg(
            Arg::new("absolute-names")
                .short('p')
                .long("absolute-names")
                .help("Do not strip leading slash characters"),
        )
        .arg(
            Arg::new("numeric")
                .short('n')
                .long("numeric")
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
