// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::path::PathBuf;

use pretty_assertions::assert_eq;
use uutests::new_ucmd;

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
    let tmp_file = create_temp_file();
    let res = new_ucmd!().arg(&tmp_file).succeeds();
    let stdout = res.no_stderr().stdout_str();

    assert!(stdout.contains(format!("# file: {}", &tmp_file.display()).as_str()));
    assert!(stdout.contains("# owner:"));
    assert!(stdout.contains("# group:"));
    assert!(stdout.contains("user:"));
    assert!(stdout.contains("group:"));
    assert!(stdout.contains("other:"));

    delete_file(&tmp_file)
}

#[test]
fn test_basic_output_with_file_with_omit_header() {
    // Create basic file with default permissions. No ACLs.
    let tmp_file = create_temp_file();
    let res = new_ucmd!().arg(&tmp_file).arg("-c").succeeds();
    let stdout = res.no_stderr().stdout_str();

    assert_eq!(
        stdout.contains(format!("# file: {}", &tmp_file.display()).as_str()),
        false
    );
    assert_eq!(stdout.contains("# owner:"), false);
    assert_eq!(stdout.contains("# group:"), false);
    assert!(stdout.contains("user:"));
    assert!(stdout.contains("group:"));
    assert!(stdout.contains("other:"));

    delete_file(&tmp_file)
}

fn get_random_number() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        .to_string()
}

fn create_temp_file() -> PathBuf {
    let tmp_dir = std::env::temp_dir();
    let tmp_file = tmp_dir.join(format!("tmp_{}", get_random_number()));
    std::fs::File::create(&tmp_file).unwrap();
    tmp_file
}

fn delete_file(tmp_file: &PathBuf) {
    std::fs::remove_file(tmp_file).unwrap();
}
