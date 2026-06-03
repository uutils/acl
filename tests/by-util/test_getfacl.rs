// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use pretty_assertions::assert_eq;
use uutests::util::TestScenario;
use uutests::{new_ucmd, util_name};

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_basic_output_without_file() {
    let res = new_ucmd!().succeeds();
    let stdout = res.no_stderr().stdout_str();

    // getfacl command shows no output if not given a file
    assert_eq!(stdout.len(), 0);
}

#[test]
fn test_basic_output_with_file() {
    // Create basic file with default permissions. No ACLs.
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("test_file");

    let res = scene.ucmd().arg("test_file").succeeds();
    let stdout = res.no_stderr().stdout_str();

    assert!(stdout.contains("# file: test_file"));
    assert!(stdout.contains("# owner:"));
    assert!(stdout.contains("# group:"));
    assert!(stdout.contains("user:"));
    assert!(stdout.contains("group:"));
    assert!(stdout.contains("other:"));
}

#[test]
fn test_basic_output_with_file_with_omit_header() {
    // Create basic file with default permissions. No ACLs.
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("test_file");

    let res = scene.ucmd().arg("test_file").arg("-c").succeeds();
    let stdout = res.no_stderr().stdout_str();

    assert_eq!(stdout.contains("# file: test_file"), false);
    assert_eq!(stdout.contains("# owner:"), false);
    assert_eq!(stdout.contains("# group:"), false);
    assert!(stdout.contains("user:"));
    assert!(stdout.contains("group:"));
    assert!(stdout.contains("other:"));
}
