#[macro_use]
extern crate derive_builder;

pub mod client;
pub mod component;
pub mod dom;
pub mod html;
pub mod init;

#[cfg(test)]
#[cfg(feature = "apply")]
mod regressions;

pub mod prelude {
    pub use crate::html::{IntoWrappedBodyNode, WrappedBodyNode};
    pub use malvolio::prelude::*;
}
