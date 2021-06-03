#[macro_use]
extern crate derive_builder;

pub mod client;
pub mod dom;
pub mod html;
pub mod init;

pub mod prelude {
    pub use crate::html::{IntoWrappedBodyNode, WrappedBodyNode};
    pub use malvolio::prelude::*;
}
