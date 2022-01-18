//! Create HTML elements. Unfortunately at the moment this is the API – in the future we'll provide
//! something a bit nicer to work with.

use std::{borrow::Cow, collections::HashMap};

use super::listener::ListenerRef;

pub mod diff;
pub mod render;

/// An HTML element.
#[derive(Builder, Clone, Default, Debug, Eq, PartialEq)]
pub struct Element {
    /// The id includes the ID of this element and all the parent elements.
    pub id: usize,
    #[builder(setter(into))]
    pub name: Cow<'static, str>,
    pub attributes: HashMap<Cow<'static, str>, Cow<'static, str>>,
    pub listeners: Vec<ListenerRef>,
    pub children: Vec<Element>,
    pub text: Option<Cow<'static, str>>,
    pub key: Option<String>,
}

impl Element {
    /// Return the unique ID of this element.
    fn id(&self) -> String {
        self.id.to_string()
    }

    /// This function returns a builder type for this struct.
    pub fn build() -> ElementBuilder {
        ElementBuilder::default()
    }
}
