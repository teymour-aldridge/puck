use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[cfg(feature = "apply")]
pub mod apply;

/// Javascript-friendly instruction serializer.
pub(crate) mod instruction_serializer;

/// A list of operations to apply to the DOM. Note that the order in which the operations are
/// applied to each element is significant.
#[derive(Debug, Clone, Serialize)]
pub struct Changeset<'a> {
    /// A DOM operation. This can be relied on to apply the operations sequentially.
    pub(crate) ops: Vec<Op<'a>>,
}

impl<'a> Changeset<'a> {
    pub(crate) fn empty() -> Self {
        Self { ops: Vec::new() }
    }
    pub(crate) fn from_op(op: Op<'a>) -> Changeset<'a> {
        Self { ops: vec![op] }
    }
    pub(crate) fn extend(&mut self, other: Changeset<'a>) {
        self.ops.extend(other.ops);
    }
}

impl<'a> IntoIterator for Changeset<'a> {
    type Item = Op<'a>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.into_iter()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Op<'a> {
    pub(crate) id: String,
    pub(crate) instruction: Instruction<'a>,
}

/// An instruction to update the DOM
#[derive(Debug, Clone, Serialize)]
pub(crate) enum Instruction<'a> {
    #[allow(unused)]
    InsertChild {
        new_child_id: String,
    },
    InsertAfter {
        after_id: String,
    },
    InsertBefore {
        before_id: String,
    },
    SetAttribute {
        key: Cow<'a, Cow<'static, str>>,
        value: Cow<'a, Cow<'static, str>>,
    },
    RemoveAttribute {
        key: Cow<'a, Cow<'static, str>>,
    },
    SetId {
        value: String,
    },
    SetText {
        value: Cow<'a, Cow<'a, str>>,
    },
    SetTagName {
        name: Cow<'a, Cow<'static, str>>,
    },
    CreateTag {
        name: Cow<'a, Cow<'static, str>>,
        parent_id: Option<String>,
    },
    RemoveText,
    RemoveListeners,
    AttachListener {
        name: String,
        on: String,
    },
    #[allow(unused)]
    SetInnerHtml {
        element: &'static str,
        html: String,
    },
}

#[derive(Debug, Deserialize)]
/// An owned analogue of `Changeset` which can be deserialized.
///
/// Mostly useful for testing.
pub struct DeserializeChangeset {
    pub ops: Vec<DeserializeOp>,
}

#[derive(Debug, Deserialize)]
/// An owned analogue of `Op` which can be deserialized.
///
/// Mostly useful for testing.
pub struct DeserializeOp {
    id: String,
    instruction: DeserializeInstruction,
}

#[derive(Debug, Deserialize)]
/// An owned analogue of `Instruction` which can be deserialized.
///
/// Mostly useful for testing.
pub enum DeserializeInstruction {
    InsertChild { new_child_id: String },
    InsertAfter { after_id: String },
    SetAttribute { key: String, value: String },
    SetId { value: String },
    SetText { value: String },
    SetTagName { name: String },
    CreateTag { name: String, parent_id: String },
    RemoveText,
    RemoveListeners,
    AttachListener { name: String, on: String },
    SetInnerHtml { element: String, html: String },
}
