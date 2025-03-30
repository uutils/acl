// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;

pub const TESTS_BINARY: &str = env!("CARGO_BIN_EXE_acl");

// Use the ctor attribute to run this function before any tests
#[ctor::ctor]
fn init() {
    unsafe {
        // Necessary for uutests to be able to find the binary
        std::env::set_var("UUTESTS_BINARY_PATH", TESTS_BINARY);
    }
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
