pub(crate) mod changeset;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod test_diffing;

use std::borrow::Cow;

use self::changeset::{Changeset, Instruction, Op};

use super::Element;

impl Element {
    /// Compare two root nodes and produce a set of instructions which can be followed to produce
    /// one from the other.
    pub fn diff<'a>(&'a self, other: Option<&'a Element>) -> Changeset<'a> {
        let mut c = Changeset::empty();

        if let Some(other) = other {
            // we need to apply all these updates...
            c.ops.extend(self.diff_name(other));
            c.ops.extend(self.diff_attributes(other));
            c.ops.extend(self.diff_text(other));
            c.ops.extend(self.diff_listeners(other));
            // ... before we change the id
            c.ops.extend(self.diff_id(other));
            c.ops.extend(self.diff_children(other));
        } else {
            c.ops.extend(self.create_from_scratch(None))
        }

        c
    }

    /// Create a new element from scratch – generally used for the first application paint.
    ///
    /// In the future we should just send this as a serialized HTML string, rather than a series of
    /// Javascript instructions.
    fn create_from_scratch(&self, parent: Option<&Element>) -> Changeset {
        let mut cs = Changeset::empty();

        cs.ops.push(Op {
            id: self.id(),
            instruction: Instruction::CreateTag {
                name: Cow::Borrowed(&self.name),
                parent_id: parent.map(|parent| parent.id()),
            },
        });

        for (key, value) in self.attributes.iter() {
            cs.ops.push(Op {
                id: self.id(),
                instruction: Instruction::SetAttribute {
                    key: Cow::Borrowed(key),
                    value: Cow::Borrowed(value),
                },
            })
        }

        cs.extend(self.generate_listeners());

        if let Some(ref value) = self.text {
            cs.ops.push(Op {
                id: self.id(),
                instruction: Instruction::SetText {
                    value: Cow::Borrowed(value),
                },
            })
        }

        for child in self.children.iter() {
            cs.extend(child.create_from_scratch(Some(self)));
        }

        cs
    }

    /// Compares the text of two nodes, and emits the relevant update needed to make sure that the
    /// text is the same.
    fn diff_text<'a>(&'a self, other: &'a Element) -> Changeset<'a> {
        dbg!(&self.text);
        dbg!(&other.text);
        let mut cs = Changeset::empty();

        if self.text != other.text {
            match other.text {
                Some(ref value) => cs.ops.push(Op {
                    id: self.id(),
                    instruction: Instruction::SetText {
                        value: Cow::Borrowed(value),
                    },
                }),
                None => cs.ops.push(Op {
                    id: self.id(),
                    instruction: Instruction::RemoveText,
                }),
            }
        }

        cs
    }

    /// Compares the ID's of two elements and emits instructions to update the underlying DOM node
    /// so that it has the new ID.
    fn diff_id<'a>(&'a self, other: &'a Element) -> Changeset {
        if self.id != other.id {
            Changeset::from_op(Op {
                id: self.id(),
                instruction: Instruction::SetId { value: other.id() },
            })
        } else {
            Changeset::empty()
        }
    }

    /// Compares the tag names of two DOM nodes and emits instructions to update the underlying DOM
    /// so that the old tag takes on the name of the new tag.
    fn diff_name<'a>(&'a self, other: &'a Element) -> Changeset<'a> {
        if self.name != other.name {
            Changeset::from_op(Op {
                id: other.id(),
                instruction: Instruction::SetTagName {
                    name: Cow::Borrowed(&other.name),
                },
            })
        } else {
            Changeset::empty()
        }
    }

    /// Compares the attributes of two DOM nodes and emits instructions to update the old DOM node
    /// to take on the attributes of the new DOM node.
    fn diff_attributes<'a>(&'a self, other: &'a Element) -> Changeset<'a> {
        let mut c = Changeset::empty();

        if self.attributes == other.attributes {
            return Changeset::empty();
        }

        for key in self.attributes.keys() {
            if !other.attributes.contains_key(key) {
                c.ops.push(Op {
                    id: self.id(),
                    instruction: Instruction::RemoveAttribute {
                        key: Cow::Borrowed(key),
                    },
                })
            }
        }

        for (their_key, their_value) in other.attributes.iter() {
            let my_value = self.attributes.get(their_key);

            if let Some(my_value) = my_value {
                if my_value != their_value {
                    c.ops.push(Op {
                        id: self.id(),
                        instruction: Instruction::SetAttribute {
                            key: Cow::Borrowed(their_key),
                            value: Cow::Borrowed(their_value),
                        },
                    })
                }
            } else {
                c.ops.push(Op {
                    id: self.id(),
                    instruction: Instruction::SetAttribute {
                        key: Cow::Borrowed(their_key),
                        value: Cow::Borrowed(their_value),
                    },
                })
            }
        }

        c
    }

    /// Compare the listeners attached to this element, and the other element, and emit instructions
    /// to modify them as needed.
    fn diff_listeners<'a>(&'a self, other: &'a Element) -> Changeset<'a> {
        let mut c = Changeset::empty();

        if self.listeners.len() != other.listeners.len() {
            c.ops.push(Op {
                id: self.id(),
                instruction: Instruction::RemoveListeners,
            });
            c.ops.extend(self.generate_listeners())
        }

        c
    }

    /// Generates all the listeners which should be attached to a given `Element`.
    fn generate_listeners(&'_ self) -> Changeset<'_> {
        let mut c = Changeset::empty();

        for listener in self.listeners.iter() {
            c.ops.push(Op {
                id: self.id(),
                instruction: Instruction::AttachListener {
                    name: listener.listener_name().to_string(),
                    on: listener.js_event().to_string(),
                },
            })
        }

        c
    }

    /// Tries to find a child with the provided key. Returns `None` if a child cannot be found.
    fn locate_child_by_key(&self, key: &str) -> Option<&Element> {
        self.children
            .iter()
            .find(|el| el.key.as_ref().unwrap().eq(key))
    }

    /// Compares the children of two nodes and emits instructions to update the old children to
    /// assume the form of the new children.
    fn diff_children<'a>(&'a self, other: &'a Element) -> Changeset<'a> {
        let mut changeset = Changeset::empty();

        if self.children_are_all_keyed() && other.children_are_all_keyed() {
            let mut unpaired = vec![];
            for (i, their_child) in other.children.iter().enumerate() {
                if let Some(my_child) = self.locate_child_by_key(their_child.key.as_ref().unwrap())
                {
                    changeset.ops.extend(my_child.diff(Some(their_child)));
                } else {
                    unpaired.push(i);
                }
            }

            let len = unpaired.len();
            for each in unpaired {
                changeset.ops.push(Op {
                    id: other.children[each].id(),
                    instruction: if each == 0 {
                        Instruction::InsertBefore {
                            before_id: other.children[1].id(),
                        }
                    } else if each == len {
                        Instruction::InsertAfter {
                            after_id: other.children[each].id(),
                        }
                    } else {
                        Instruction::InsertBefore {
                            before_id: other.children[each - 1].id(),
                        }
                    },
                })
            }
        } else {
            let self_len = self.children.len();
            let other_len = other.children.len();
            for (my_child, their_child) in self.children.iter().zip(&other.children) {
                changeset.ops.extend(my_child.diff(Some(their_child)));
            }

            if self_len != other_len {
                let (create_elements_from, start_index) = if self_len > other_len {
                    (&self.children, other_len)
                } else {
                    (&other.children, self_len)
                };

                for el in create_elements_from[start_index..].iter() {
                    changeset.ops.extend(el.create_from_scratch(Some(other)));
                }
            }
        }

        changeset
    }

    /// Checks that the `Element` doesn't have children without a key.
    fn children_are_all_keyed(&self) -> bool {
        !self.children.iter().any(|el| el.key.is_none())
    }
}
