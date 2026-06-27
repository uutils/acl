// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;

pub const TESTS_BINARY: &str = env!("CARGO_BIN_EXE_acl");

// Use the ctor attribute to run this function before any tests
#[ctor::ctor(unsafe)]
fn init() {
    unsafe {
        // Necessary for uutests to be able to find the binary
        std::env::set_var("UUTESTS_BINARY_PATH", TESTS_BINARY);
    }
}

/// Regression test to ensure the acl binary handles write errors gracefully
/// when stdout is redirected to /dev/full instead of panicking.
///
/// This test verifies the fix for the issue where the binary would panic with:
/// "thread 'main' panicked at failed printing to stdout: No space left on device"
///
/// Expected behavior:
/// - The command should NOT panic (exit code 134/SIGABRT)
/// - The command should handle the write error gracefully
#[test]
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
fn test_acl_binary_write_error_no_panic() {
    use std::fs::OpenOptions;
    use std::process::Command;

    // Open /dev/full for stdout redirection
    let dev_full = OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("Failed to open /dev/full");

    // Run the acl binary (with no arguments, which prints usage) with stdout redirected to /dev/full
    let result = Command::new(TESTS_BINARY)
        .stdout(dev_full)
        .status()
        .expect("Failed to execute acl binary");

    // The command should NOT panic with SIGABRT (exit code 134)
    // A panic would cause SIGABRT which typically results in exit code 134
    let code = result.code().unwrap_or(0);
    assert_ne!(
        code, 134,
        "acl binary panicked when writing to /dev/full (exit code 134/SIGABRT). \
         This means println! macros or .unwrap()/.expect() on write operations are still being used."
    );

    // If we get here without panicking, the test passes
}

#[cfg(feature = "chacl")]
#[path = "by-util/test_chacl.rs"]
mod test_chacl;

#[cfg(feature = "getfacl")]
#[path = "by-util/test_getfacl.rs"]
mod test_getfacl;

#[cfg(feature = "setfacl")]
#[path = "by-util/test_setfacl.rs"]
mod test_setfacl;
