// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
use clap::{Arg, ArgAction, Command, crate_version};
use uucore::{error::UResult, help_about, help_usage};

const ABOUT: &str = help_about!("chacl.md");
const USAGE: &str = help_usage!("chacl.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // Example: Handle pathname arguments
    if let Some(pathnames) = matches.get_many::<String>("pathname") {
        for pathname in pathnames {
            println!("Would change ACL for pathname: {pathname}");
            // Here you would implement the logic to change the ACLs
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(USAGE)
        .arg(Arg::new("acl")
             .help("ACL string to apply")
             .required(false)
             .action(ArgAction::Set)
             .value_name("ACL"))
        .arg(Arg::new("b")
             .short('b')
             .help("Indicates that there are two ACLs to change, the first is the file access ACL and the second the directory default ACL")
             .action(ArgAction::Set)
             .value_name("ACL_DACL"))
        .arg(Arg::new("d")
             .short('d')
             .help("Used to set only the default ACL of a directory")
             .action(ArgAction::Set)
             .value_name("DACL"))
        .arg(Arg::new("R")
             .short('R')
             .help("Removes the file access ACL only"))
        .arg(Arg::new("D")
             .short('D')
             .help("Removes directory default ACL only"))
        .arg(Arg::new("B")
             .short('B')
             .help("Remove all ACLs"))
        .arg(Arg::new("l")
             .short('l')
             .help("Lists the access ACL and possibly the default ACL associated with the specified files or directories"))
        .arg(Arg::new("r")
             .short('r')
             .help("Set the access ACL recursively for each subtree rooted at pathname(s)"))
        .arg(Arg::new("pathname")
             .help("Pathname to change ACLs for")
             .required(false)
             .action(ArgAction::Append)
             .value_name("PATHNAME"))
}
