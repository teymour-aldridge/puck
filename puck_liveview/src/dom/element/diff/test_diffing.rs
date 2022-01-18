use std::collections::HashMap;

use crate::dom::{element::Element, listener::ListenerRef};

#[test]
fn test_name_change() {
    let old = Element {
        id: 0,
        name: "div".into(),
        ..Default::default()
    };

    let new = Element {
        id: 0,
        name: "p".into(),
        ..Default::default()
    };

    let cs = old.diff(Some(&new));

    insta::assert_debug_snapshot!("test_name_change", cs);
}

#[test]
fn test_single_element_attribute_change() {
    let old = Element {
        id: 0,
        name: "div".into(),
        attributes: {
            let mut res = HashMap::new();
            res.insert("class".into(), "one".into());
            res.insert(
                "attribute-which-doesn-t-exist-after-diffing".into(),
                "1".into(),
            );
            res
        },
        ..Default::default()
    };

    let new = Element {
        id: 0,
        name: "div".into(),
        attributes: {
            let mut res = HashMap::new();
            res.insert("class".into(), "two".into());
            res.insert("new-attribute-added-after-diffing".into(), "value".into());
            res
        },
        ..Default::default()
    };

    let cs = old.diff(Some(&new));

    assert_eq!(cs.ops.len(), 3);
    assert!(cs.ops.iter().any(|op| {
        match &op.instruction {
            crate::dom::element::diff::changeset::Instruction::SetAttribute { key, value } => {
                key.to_string() == "new-attribute-added-after-diffing".to_string()
                    && value.to_string() == "value".to_string()
            }
            _ => false,
        }
    }));
    assert!(cs.ops.iter().any(|op| {
        match &op.instruction {
            crate::dom::element::diff::changeset::Instruction::SetAttribute { key, value } => {
                key.to_string() == "class".to_string() && value.to_string() == "two".to_string()
            }
            _ => false,
        }
    }));
}

#[test]
fn test_single_element_text_change() {
    let old = Element {
        id: 0,
        name: "p".into(),
        text: Some("the cat sat on the mat".into()),
        ..Default::default()
    };

    let new = Element {
        id: 0,
        name: "p".into(),
        text: Some("the mat sat on the cat".into()),
        ..Default::default()
    };

    let c = old.diff(Some(&new));
    insta::assert_debug_snapshot!("test_single_element_text_change", c);
}

#[test]
fn test_add_child_change() {
    let old = Element {
        id: 0,
        name: "div".into(),
        children: vec![Element {
            id: 3,
            key: Some("a".into()),
            name: "p".into(),
            text: Some("the cat sat on the mat".into()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let new = Element {
        id: 0,
        name: "div".into(),
        children: vec![
            Element {
                id: 3,
                name: "p".into(),
                key: Some("a".into()),
                text: Some("the cat sat on the mat".into()),
                ..Default::default()
            },
            Element {
                id: 2,
                name: "p".into(),
                key: Some("b".into()),
                text: Some("the mat sat on the cat".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let c = old.diff(Some(&new));
    insta::assert_debug_snapshot!("test_add_child_change", c);
}

#[test]
fn test_add_child_before_change() {
    let old = Element {
        id: 0,
        name: "div".into(),
        children: vec![Element {
            id: 3,
            key: Some("a".into()),
            name: "p".into(),
            text: Some("the cat sat on the mat".into()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let new = Element {
        id: 0,
        name: "div".into(),
        children: vec![
            Element {
                id: 3,
                name: "p".into(),
                key: Some("b".into()),
                text: Some("the mat sat on the cat".into()),
                ..Default::default()
            },
            Element {
                id: 2,
                name: "p".into(),
                key: Some("a".into()),
                text: Some("the cat sat on the mat".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let c = old.diff(Some(&new));
    insta::assert_debug_snapshot!("test_add_child_before_change", c);
}

#[test]
fn test_more_complex_diff() {
    let old = Element {
        id: 0,
        name: std::borrow::Cow::Borrowed("div"),
        attributes: {
            let _cap = <[()]>::len(&[()]);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            let _ = _map.insert(("class".into()), ("message-list".into()));
            _map
        },
        listeners: vec![],
        children: vec![
            Element {
                id: 3,
                name: std::borrow::Cow::Borrowed("div"),
                attributes: HashMap::new(),
                listeners: vec![],
                children: vec![Element {
                    id: 4,
                    name: std::borrow::Cow::Borrowed("input"),
                    attributes: HashMap::new(),
                    listeners: vec![ListenerRef::new("msg-input", "input")],
                    children: vec![],
                    text: None,
                    key: None,
                }],
                text: None,
                key: None,
            },
            Element {
                id: 2,
                name: std::borrow::Cow::Borrowed("div"),
                attributes: HashMap::new(),
                listeners: vec![],
                children: vec![Element {
                    id: 5,
                    name: std::borrow::Cow::Borrowed("button"),
                    attributes: HashMap::new(),
                    listeners: vec![ListenerRef::new("msg-submit", "click")],
                    children: vec![],
                    text: Some(std::borrow::Cow::Borrowed("Send message")),
                    key: None,
                }],
                text: None,
                key: None,
            },
        ],
        text: None,
        key: None,
    };
    let new = Element {
        id: 0,
        name: std::borrow::Cow::Borrowed("div"),
        attributes: {
            let _cap = <[()]>::len(&[()]);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            let _ = _map.insert(("class".into()), ("message-list".into()));
            _map
        },
        listeners: vec![],
        children: vec![
            Element {
                id: 3,
                name: std::borrow::Cow::Borrowed("div"),
                attributes: HashMap::new(),
                listeners: vec![],
                children: vec![Element {
                    id: 4,
                    name: std::borrow::Cow::Borrowed("input"),
                    attributes: HashMap::new(),
                    listeners: vec![ListenerRef::new(
                        "msg-input".to_string(),
                        "input".to_string(),
                    )],
                    children: vec![],
                    text: None,
                    key: None,
                }],
                text: None,
                key: None,
            },
            Element {
                id: 2,
                name: std::borrow::Cow::Borrowed("div"),
                attributes: HashMap::new(),
                listeners: vec![],
                children: vec![Element {
                    id: 5,
                    name: std::borrow::Cow::Borrowed("button"),
                    attributes: HashMap::new(),
                    listeners: vec![ListenerRef::new(
                        "msg-submit".to_string(),
                        "click".to_string(),
                    )],
                    children: vec![],
                    text: Some(std::borrow::Cow::Borrowed("Send message")),
                    key: None,
                }],
                text: None,
                key: None,
            },
            Element {
                id: 9,
                name: std::borrow::Cow::Borrowed("div"),
                attributes: {
                    let _cap = <[()]>::len(&[()]);
                    let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                    let _ = _map.insert(("class".into()), ("message-container".into()));
                    _map
                },
                listeners: vec![],
                children: vec![
                    Element {
                        id: 8,
                        name: std::borrow::Cow::Borrowed("p"),
                        attributes: {
                            let _cap = <[()]>::len(&[()]);
                            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                            let _ = _map.insert(("class".into()), ("message-sent-at".into()));
                            _map
                        },
                        listeners: vec![],
                        children: vec![],
                        text: Some(std::borrow::Cow::Borrowed("1970-01-25 06:34:13")),
                        key: None,
                    },
                    Element {
                        id: 6,
                        name: std::borrow::Cow::Borrowed("p"),
                        attributes: {
                            let _cap = <[()]>::len(&[()]);
                            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                            let _ = _map.insert(("class".into()), ("message-author".into()));
                            _map
                        },
                        listeners: vec![],
                        children: vec![],
                        text: Some(std::borrow::Cow::Borrowed("[username not set]")),
                        key: None,
                    },
                    Element {
                        id: 7,
                        name: std::borrow::Cow::Borrowed("p"),
                        attributes: {
                            let _cap = <[()]>::len(&[()]);
                            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                            let _ = _map.insert(("class".into()), ("message-contents".into()));
                            _map
                        },
                        listeners: vec![],
                        children: vec![],
                        text: Some(std::borrow::Cow::Borrowed("sending message")),
                        key: None,
                    },
                ],
                text: None,
                key: None,
            },
        ],
        text: None,
        key: None,
    };

    let c = old.diff(Some(&new));
    insta::assert_debug_snapshot!(c);
}

#[test]
fn test_delete_child_change() {}
