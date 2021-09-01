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
    ///
    /// This might need to be changed in the future.
    pub id: Vec<u32>,
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
    ///
    /// At the moment this allocates way too much – at some point we should update this structure
    /// to use more indirection.
    fn id(&self) -> String {
        self.id
            .iter()
            .map(ToString::to_string)
            .map(|x| x + "-")
            .collect::<String>()
    }

    /// This function returns a builder type for this struct.
    pub fn build() -> ElementBuilder {
        ElementBuilder::default()
    }

    /// Create an `Element` using `Default::default()`, except for the API specified.
    pub fn default_with_id(id: Vec<u32>) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}
