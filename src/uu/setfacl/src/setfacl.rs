// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{Arg, ArgAction, Command, crate_version};
use uucore::{error::UResult, help_about, help_usage};

const ABOUT: &str = help_about!("setfacl.md");
const USAGE: &str = help_usage!("setfacl.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Here, you would add the logic to handle each option and modify ACLs accordingly.
    // This example focuses on setting up the command-line interface.

    println!("setfacl options selected:");
    for option in matches.get_many::<String>("option").unwrap_or_default() {
        println!("Option: {}", option);
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(USAGE)
        .arg(
            Arg::new("modify")
                .short('m')
                .long("modify")
                .help("Modify ACL of a file or directory")
                .action(ArgAction::Append)
                .value_name("ACL_SPEC"),
        )
        .arg(
            Arg::new("remove")
                .short('x')
                .long("remove")
                .help("Remove ACL entries")
                .action(ArgAction::Append)
                .value_name("ACL_SPEC"),
        )
        .arg(
            Arg::new("file")
                .help("File to modify ACLs for")
                .required(true)
                .action(ArgAction::Append)
                .value_name("FILE"),
        )
        .arg(
            Arg::new("remove-all")
                .short('b')
                .long("remove-all")
                .help("Remove all extended ACL entries"),
        )
        .arg(
            Arg::new("remove-default")
                .short('k')
                .long("remove-default")
                .help("Remove the Default ACL"),
        )
        .arg(
            Arg::new("no-mask")
                .short('n')
                .long("no-mask")
                .help("Do not recalculate the effective rights mask"),
        )
        .arg(Arg::new("mask").long("mask").help(
            "Recalculate the effective rights mask, even if a mask entry was explicitly given",
        ))
        .arg(
            Arg::new("default")
                .short('d')
                .long("default")
                .help("Operations apply to the Default ACL"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .help("Apply operations to all files and directories recursively"),
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
            Arg::new("restore")
                .long("restore")
                .help("Restore a permission backup created by getfacl -R or similar")
                .value_name("FILE"),
        )
        .arg(Arg::new("test").long("test").help(
            "Test mode. Instead of changing the ACLs of any files, the resulting ACLs are listed",
        ))
        .arg(
            Arg::new("option")
                .action(ArgAction::Set)
                .num_args(0..)
                .last(true),
        )
}
