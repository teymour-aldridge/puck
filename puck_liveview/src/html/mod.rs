use malvolio::prelude::BodyNode;
use maplit::hashmap;

use crate::dom::{element::Element, listener::ListenerRef};

#[derive(Debug)]
pub struct WrappedBodyNode {
    node: BodyNode,
    listeners: Vec<ListenerRef>,
    children: Vec<WrappedBodyNode>,
}

macro_rules! map_heading_to_element {
    ($self:ident, $id:ident, $h:ident) => {{
        let $h = $h.into_pub_fields();
        Element {
            id: $id,
            name: std::borrow::Cow::Borrowed(stringify!($h)),
            attributes: $h.attrs,
            listeners: $self.listeners,
            // headings currently can't have children; this will be rectified in the future
            children: vec![],
            text: Some($h.text),
            key: None,
        }
    }};
}

impl WrappedBodyNode {
    pub fn into_element(self, id: Vec<u32>) -> Element {
        let children_mapper = |(index, child): (usize, WrappedBodyNode)| {
            let mut id = id.clone();
            id.push(index as u32);
            child.into_element(id)
        };

        match self.node {
            BodyNode::H1(h1) => {
                map_heading_to_element!(self, id, h1)
            }
            BodyNode::H2(h2) => {
                map_heading_to_element!(self, id, h2)
            }
            BodyNode::H3(h3) => {
                map_heading_to_element!(self, id, h3)
            }
            BodyNode::H4(h4) => {
                map_heading_to_element!(self, id, h4)
            }
            BodyNode::H5(h5) => {
                map_heading_to_element!(self, id, h5)
            }
            BodyNode::H6(h6) => {
                map_heading_to_element!(self, id, h6)
            }
            BodyNode::P(p) => {
                let p = p.into_pub_fields();
                let mut children = vec![];
                let mut text = None;
                for each in p.children {
                    if let BodyNode::Text(t) = each {
                        text = Some(t.into_pub_fields().text);
                    } else {
                        children.push(each);
                    }
                }
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("p"),
                    attributes: p.attrs,
                    listeners: self.listeners,
                    children: self
                        .children
                        .into_iter()
                        .enumerate()
                        .map(children_mapper)
                        .collect(),
                    text,
                    key: None,
                }
            }
            BodyNode::Form(form) => {
                let form = form.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("form"),
                    attributes: form.attrs,
                    listeners: self.listeners,
                    children: self
                        .children
                        .into_iter()
                        .enumerate()
                        .map(children_mapper)
                        .collect(),
                    text: None,
                    key: None,
                }
            }
            BodyNode::Br(_) => Element {
                id,
                name: std::borrow::Cow::Borrowed("br"),
                attributes: hashmap! {},
                listeners: vec![],
                children: vec![],
                text: None,
                key: None,
            },
            BodyNode::Div(div) => {
                let div = div.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("div"),
                    attributes: div.attrs,
                    listeners: self.listeners,
                    children: self
                        .children
                        .into_iter()
                        .enumerate()
                        .map(children_mapper)
                        .collect(),
                    text: None,
                    key: None,
                }
            }
            BodyNode::A(a) => {
                let a = a.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("a"),
                    attributes: a.attrs,
                    listeners: self.listeners,
                    children: vec![],
                    text: Some(a.text),
                    key: None,
                }
            }
            BodyNode::Input(input) => {
                let input = input.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("input"),
                    attributes: input.attrs,
                    listeners: self.listeners,
                    children: self
                        .children
                        .into_iter()
                        .enumerate()
                        .map(children_mapper)
                        .collect(),
                    text: None,
                    key: None,
                }
            }
            BodyNode::Label(label) => {
                map_heading_to_element!(self, id, label)
            }
            BodyNode::Select(select) => {
                let select = select.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("div"),
                    attributes: select.attrs,
                    listeners: self.listeners,
                    children: self
                        .children
                        .into_iter()
                        .enumerate()
                        .map(children_mapper)
                        .collect(),
                    text: None,
                    key: None,
                }
            }
            // not very useful given that Puck is entirely dependent on Javascript, but hey.
            BodyNode::NoScript(noscript) => {
                let noscript = noscript.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("div"),
                    attributes: hashmap! {},
                    listeners: vec![],
                    children: vec![],
                    text: Some(noscript.text),
                    key: None,
                }
            }
            BodyNode::Text(_) => panic!(""),
            BodyNode::Img(img) => {
                let img = img.into_pub_fields();
                Element {
                    id: id.clone(),
                    name: std::borrow::Cow::Borrowed("img"),
                    attributes: img.attrs,
                    listeners: self.listeners,
                    children: vec![],
                    text: None,
                    key: None,
                }
            }
        }
    }

    pub fn listener(mut self, listener: impl Into<ListenerRef>) -> Self {
        self.listeners.push(listener.into());
        self
    }

    pub fn child(mut self, child: impl Into<WrappedBodyNode>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = WrappedBodyNode>) -> Self {
        self.children.extend(children);
        self
    }
}

pub trait IntoWrappedBodyNode {
    fn wrap(self) -> WrappedBodyNode;
}

impl<T> IntoWrappedBodyNode for T
where
    T: Into<BodyNode>,
{
    fn wrap(self) -> WrappedBodyNode {
        WrappedBodyNode {
            node: self.into(),
            listeners: vec![],
            children: vec![],
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[test]
fn test_html_conversion() {
    use malvolio::prelude::*;

    let tree = Div::new();
    let output = tree.wrap().into_element(vec![0]);
    insta::assert_debug_snapshot!("html_conversion_simple", output);

    let bigger_tree = Div::new().wrap().child(H1::new("Heading 1").wrap()).child(
        Input::new()
            .attribute(Type::Submit)
            .wrap()
            .listener(ListenerRef::new("a_listener", "click")),
    );
    let output = bigger_tree.into_element(vec![0]);
    insta::assert_debug_snapshot!("html_conversion_medium", output);

    let id_not_starting_from_zero = Form::new()
        .wrap()
        .child(Input::new().attribute(Type::Text).wrap())
        .child(Input::new().attribute(Type::Submit).wrap());
    let output = id_not_starting_from_zero.into_element(vec![0, 0, 0]);
    insta::assert_debug_snapshot!("html_conversion_offset_starting_id", output);
}
