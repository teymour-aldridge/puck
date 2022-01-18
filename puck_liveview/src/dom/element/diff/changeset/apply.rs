use std::borrow::Cow;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::dom::element::Element;
use crate::dom::listener::ListenerRef;

use super::Changeset;

impl<'a> Changeset<'a> {
    /// Apply the changeset to an `Element` in-place.
    ///
    /// **This method is only available if you have activated the `apply` feature.**
    ///
    /// This method is probably not useful to you – it is here for testing purposes.
    pub fn apply(&self, element: &mut Element) {
        for op in self.ops.iter() {
            let el_id = Self::parse(op.id.clone());
            match &op.instruction {
                super::Instruction::InsertChild { new_child_id } => {
                    let el =
                        Self::find_el_with_id(el_id, element).expect("failed to find the element");

                    el.children.push(Element {
                        id: Self::parse(new_child_id),
                        ..Default::default()
                    })
                }
                super::Instruction::InsertAfter { after_id } => {
                    let el = Self::find_el_with_id(el_id, element).expect("failed to find element");

                    assert!(!el.children.is_empty());

                    let after_id = Self::parse(after_id);

                    let index = el
                        .children
                        .iter()
                        .enumerate()
                        .find_map(
                            |(index, el)| {
                                if el.id == after_id {
                                    Some(index)
                                } else {
                                    None
                                }
                            },
                        )
                        .expect("`InsertAfter` – specified node to insert after was not found in children");

                    el.children.insert(
                        index + 1,
                        Element {
                            id: after_id,
                            ..Default::default()
                        },
                    );
                }
                super::Instruction::InsertBefore { before_id } => {
                    let el = Self::find_el_with_id(el_id, element).expect("failed to find element");

                    assert!(!el.children.is_empty());

                    let before_id = Self::parse(before_id);

                    let index = el
                        .children
                        .iter()
                        .enumerate()
                        .find_map(
                            |(index, el)| {
                                if el.id == before_id {
                                    Some(index)
                                } else {
                                    None
                                }
                            },
                        )
                        .expect("`InsertAfter` – specified node to insert after was not found in children");

                    el.children.insert(
                        index,
                        Element {
                            id: before_id,
                            ..Default::default()
                        },
                    );
                }
                super::Instruction::SetAttribute { key, value } => {
                    let el = Self::find_el_with_id(el_id.clone(), element)
                        .expect("failed to find element to set attribute of");

                    el.attributes
                        .insert(key.clone().into_owned(), value.clone().into_owned());
                }
                super::Instruction::SetId { value } => {
                    Self::find_and_mutate(element, el_id, |el| {
                        el.id = Self::parse(value);
                    })
                }
                super::Instruction::SetText { value } => {
                    Self::find_and_mutate(element, el_id, |el| {
                        el.text = Some(Self::crudely_remove_cow_lifetime_problems(value));
                    })
                }
                super::Instruction::SetTagName { name } => {
                    Self::find_and_mutate(element, el_id, |el| {
                        el.name = Self::crudely_remove_cow_lifetime_problems(name);
                    })
                }
                super::Instruction::CreateTag {
                    name: _,
                    parent_id: _,
                } => {
                    panic!(
                        "this operation should not be possible to trigger (each tree has a single
                        parent, and that parent will never be deleted)"
                    )
                }
                super::Instruction::RemoveText => Self::find_and_mutate(element, el_id, |el| {
                    el.text = None;
                }),
                super::Instruction::RemoveListeners => {
                    Self::find_and_mutate(element, el_id, |el| {
                        el.listeners = vec![];
                    })
                }
                super::Instruction::AttachListener { name, on } => {
                    Self::find_and_mutate(element, el_id, |el| {
                        el.listeners.push(ListenerRef::new(name, on))
                    })
                }
                super::Instruction::SetInnerHtml {
                    element: _,
                    html: _,
                } => {
                    panic!("this method should not be called from within this test")
                }
                super::Instruction::RemoveAttribute { key } => {
                    Self::find_and_mutate(element, el_id, |el| {
                        el.attributes
                            .remove(&Self::crudely_remove_cow_lifetime_problems(key));
                    })
                }
            }
        }
    }

    fn crudely_remove_cow_lifetime_problems(cow: &Cow<'_, Cow<'_, str>>) -> Cow<'static, str> {
        cow.clone().into_owned().into_owned().into()
    }

    fn find_and_mutate(element: &mut Element, id: Vec<u32>, mutate: impl FnOnce(&mut Element)) {
        let el = Self::find_el_with_id(id, element).expect("failed to find element to mutate");
        (mutate)(el)
    }

    fn try_parse(id: impl AsRef<str>) -> Result<Vec<u32>, ParseIntError> {
        id.parse::<usize>()
    }

    fn parse(id: impl AsRef<str>) -> usize {
        Self::try_parse(id.as_ref()).expect(&format!("could not parse the id {:#?}", id.as_ref()))
    }

    /// Conducts a depth-first search through the tree for the element.
    fn find_el_with_id(id: usize, element: &mut Element) -> Option<&mut Element> {
        if id == element.id {
            return Some(element);
        }
        for each in element.children.iter_mut() {
            let el = Self::find_el_with_id(id.clone(), each);
            if el.is_some() {
                return el;
            }
        }
        None
    }
}
