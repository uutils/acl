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

#[test]
fn test_basic_output_with_file_with_access_flag() {
    // Create basic file with default permissions. No ACLs.
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("test_file");

    let res = scene.ucmd().arg("test_file").arg("-a").succeeds();
    let stdout = res.no_stderr().stdout_str();

    assert!(stdout.contains("# file: test_file"));
    assert!(stdout.contains("# owner:"));
    assert!(stdout.contains("# group:"));
    assert!(stdout.contains("user:"));
    assert!(stdout.contains("group:"));
    assert!(stdout.contains("other:"));
}

#[test]
fn test_strips_leading_slash_by_default() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("test_file");
    let abs = at.plus_as_string("test_file");
    assert!(abs.starts_with('/'));

    let res = scene.ucmd().arg(&abs).succeeds();
    res.stderr_is("getfacl: Removing leading '/' from absolute path names\n");
    let stdout = res.stdout_str();
    let expected = format!("# file: {}\n", abs.trim_start_matches('/'));
    assert!(stdout.contains(&expected), "stdout was: {stdout}");
}

#[test]
fn test_leading_slash_warning_printed_once() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("a");
    at.touch("b");

    scene
        .ucmd()
        .arg(at.plus_as_string("a"))
        .arg(at.plus_as_string("b"))
        .succeeds()
        .stderr_is("getfacl: Removing leading '/' from absolute path names\n");
}

#[test]
fn test_root_is_shown_as_dot() {
    let res = new_ucmd!().arg("/").succeeds();
    res.stderr_is("getfacl: Removing leading '/' from absolute path names\n");
    assert!(res.stdout_str().starts_with("# file: .\n"));
}

#[test]
fn test_absolute_names_keeps_leading_slash() {
    for flag in ["-p", "--absolute-names"] {
        let scene = TestScenario::new(util_name!());
        let at = &scene.fixtures;
        at.touch("test_file");
        let abs = at.plus_as_string("test_file");

        let res = scene.ucmd().arg(flag).arg(&abs).succeeds();
        let stdout = res.no_stderr().stdout_str();
        assert!(
            stdout.contains(&format!("# file: {abs}\n")),
            "stdout was: {stdout}"
        );
    }
}

#[test]
fn test_relative_name_unchanged() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("test_file");

    let res = scene.ucmd().arg("./test_file").succeeds();
    let stdout = res.no_stderr().stdout_str();
    assert!(
        stdout.contains("# file: test_file\n"),
        "stdout was: {stdout}"
    );
}

/// Little endian POSIX ACL xattr: version 2, then (tag, perm, id) entries
#[cfg(target_os = "linux")]
fn acl_xattr(entries: &[(u16, u16, u32)]) -> Vec<u8> {
    let mut value = 2u32.to_le_bytes().to_vec();
    for &(tag, perm, id) in entries {
        value.extend_from_slice(&tag.to_le_bytes());
        value.extend_from_slice(&perm.to_le_bytes());
        value.extend_from_slice(&id.to_le_bytes());
    }
    value
}

#[test]
#[cfg(target_os = "linux")]
fn test_named_entries_and_effective_rights() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.touch("test_file");
    // user::rw- user:0:r-- group::rw- group:0:rwx mask::r-- other::r--
    let acl = acl_xattr(&[
        (0x01, 6, u32::MAX),
        (0x02, 4, 0),
        (0x04, 6, u32::MAX),
        (0x08, 7, 0),
        (0x10, 4, u32::MAX),
        (0x20, 4, u32::MAX),
    ]);
    if xattr::set(at.plus("test_file"), "system.posix_acl_access", &acl).is_err() {
        // The file system has no POSIX ACL support
        return;
    }
    let user = uucore::entries::uid2usr(0).unwrap();
    let group = uucore::entries::gid2grp(0).unwrap();

    scene
        .ucmd()
        .args(&["-c", "test_file"])
        .succeeds()
        .stdout_is(format!(
            "user::rw-\nuser:{user}:r--\ngroup::rw-\t#effective:r--\ngroup:{group}:rwx\t#effective:r--\nmask::r--\nother::r--\n\n"
        ));

    scene
        .ucmd()
        .args(&["-c", "-e", "test_file"])
        .succeeds()
        .stdout_contains(format!("user:{user}:r--\t#effective:r--\n"));

    scene
        .ucmd()
        .args(&["-c", "-E", "test_file"])
        .succeeds()
        .stdout_does_not_contain("#effective");
}

#[test]
#[cfg(target_os = "linux")]
fn test_default_acl() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;
    at.mkdir("test_dir");
    at.set_mode("test_dir", 0o755);
    let acl = acl_xattr(&[
        (0x01, 7, u32::MAX),
        (0x04, 5, u32::MAX),
        (0x20, 5, u32::MAX),
    ]);
    if xattr::set(at.plus("test_dir"), "system.posix_acl_default", &acl).is_err() {
        return;
    }

    scene
        .ucmd()
        .args(&["-c", "test_dir"])
        .succeeds()
        .stdout_is(
            "user::rwx\ngroup::r-x\nother::r-x\ndefault:user::rwx\ndefault:group::r-x\ndefault:other::r-x\n\n",
        );

    scene
        .ucmd()
        .args(&["-c", "-d", "test_dir"])
        .succeeds()
        .stdout_is("user::rwx\ngroup::r-x\nother::r-x\n\n");
}
