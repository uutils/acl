// This file is part of the uutils acl package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
#[macro_use]
mod common;

#[cfg(feature = "chacl")]
#[path = "by-util/test_chacl.rs"]
mod test_chacl;

#[cfg(feature = "getfacl")]
#[path = "by-util/test_getfacl.rs"]
mod test_getfacl;

#[cfg(feature = "setfacl")]
#[path = "by-util/test_setfacl.rs"]
mod test_setfacl;
