/*!
[bindgen]: https://github.com/rust-lang/rust-bindgen
[libloading]: https://github.com/nagisa/rust_libloading
*/

// bindgen generates code that triggers these lints
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_static_lifetimes)]

// Generate bindings: ./gen_bindings.sh
//
// We avoid generating layout tests because they cause a large number of
// warnings and according to commentary are not useful. See
// https://github.com/rust-lang/rust-bindgen/issues/1651 for more.
mod bindings;
pub use self::bindings::*;
