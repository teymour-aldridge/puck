use maplit::hashmap;

use crate::dom::{element::Element, listener::ListenerRef};

#[test]
fn test_name_change() {
    let old = Element {
        id: vec![0],
        name: "div".into(),
        ..Default::default()
    };

    let new = Element {
        id: vec![0],
        name: "p".into(),
        ..Default::default()
    };

    let cs = old.diff(Some(&new));

    insta::assert_debug_snapshot!("test_name_change", cs);
}

#[test]
fn test_single_element_attribute_change() {
    let old = Element {
        id: vec![0],
        name: "div".into(),
        attributes: hashmap! {
            "class".into() => "one".into(),
            "attribute-which-doesn-t-exist-after-diffing".into() => "1".into()
        },
        ..Default::default()
    };

    let new = Element {
        id: vec![0],
        name: "div".into(),
        attributes: hashmap! {
            "class".into() => "two".into(),
            "new-attribute-added-after-diffing".into() => "value".into()
        },
        ..Default::default()
    };

    let cs = old.diff(Some(&new));

    insta::assert_debug_snapshot!("test_single_element_attribute_change", cs);
}

#[test]
fn test_single_element_text_change() {
    let old = Element {
        id: vec![0],
        name: "p".into(),
        text: Some("the cat sat on the mat".into()),
        ..Default::default()
    };

    let new = Element {
        id: vec![0],
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
        id: vec![0],
        name: "div".into(),
        children: vec![Element {
            id: vec![0, 0],
            key: Some("a".to_string()),
            name: "p".into(),
            text: Some("the cat sat on the mat".into()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let new = Element {
        id: vec![0],
        name: "div".into(),
        children: vec![
            Element {
                id: vec![0, 0],
                name: "p".into(),
                key: Some("a".to_string()),
                text: Some("the cat sat on the mat".into()),
                ..Default::default()
            },
            Element {
                id: vec![0, 1],
                name: "p".into(),
                key: Some("b".to_string()),
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
        id: vec![0],
        name: "div".into(),
        children: vec![Element {
            id: vec![0, 0],
            key: Some("a".to_string()),
            name: "p".into(),
            text: Some("the cat sat on the mat".into()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let new = Element {
        id: vec![0],
        name: "div".into(),
        children: vec![
            Element {
                id: vec![0, 0],
                name: "p".into(),
                key: Some("b".to_string()),
                text: Some("the mat sat on the cat".into()),
                ..Default::default()
            },
            Element {
                id: vec![0, 1],
                name: "p".into(),
                key: Some("a".to_string()),
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
        id: vec![0],
        name: std::borrow::Cow::Borrowed("div"),
        attributes: hashmap! {
            "class".into() => "message-list".into(),
        },
        listeners: vec![],
        children: vec![
            Element {
                id: vec![0, 0],
                name: std::borrow::Cow::Borrowed("div"),
                attributes: hashmap! {},
                listeners: vec![],
                children: vec![Element {
                    id: vec![0, 0, 0],
                    name: std::borrow::Cow::Borrowed("input"),
                    attributes: hashmap! {},
                    listeners: vec![ListenerRef::new("msg-input", "input")],
                    children: vec![],
                    text: None,
                    key: None,
                }],
                text: None,
                key: None,
            },
            Element {
                id: vec![0, 1],
                name: std::borrow::Cow::Borrowed("div"),
                attributes: hashmap! {},
                listeners: vec![],
                children: vec![Element {
                    id: vec![0, 1, 0],
                    name: std::borrow::Cow::Borrowed("button"),
                    attributes: hashmap! {},
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
        id: vec![0],
        name: std::borrow::Cow::Borrowed("div"),
        attributes: hashmap! {
            "class".into() => "message-list".into(),
        },
        listeners: vec![],
        children: vec![
            Element {
                id: vec![0, 0],
                name: std::borrow::Cow::Borrowed("div"),
                attributes: hashmap! {},
                listeners: vec![],
                children: vec![Element {
                    id: vec![0, 0, 0],
                    name: std::borrow::Cow::Borrowed("input"),
                    attributes: hashmap! {},
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
                id: vec![0, 1],
                name: std::borrow::Cow::Borrowed("div"),
                attributes: hashmap! {},
                listeners: vec![],
                children: vec![Element {
                    id: vec![0, 1, 0],
                    name: std::borrow::Cow::Borrowed("button"),
                    attributes: hashmap! {},
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
                id: vec![0, 2],
                name: std::borrow::Cow::Borrowed("div"),
                attributes: hashmap! {
                    "class".into() =>  "message-container".into(),
                },
                listeners: vec![],
                children: vec![
                    Element {
                        id: vec![0, 2, 0],
                        name: std::borrow::Cow::Borrowed("p"),
                        attributes: hashmap! {
                            "class".into() => "message-sent-at".into(),
                        },
                        listeners: vec![],
                        children: vec![],
                        text: Some(std::borrow::Cow::Borrowed("1970-01-25 06:34:13")),
                        key: None,
                    },
                    Element {
                        id: vec![0, 2, 1],
                        name: std::borrow::Cow::Borrowed("p"),
                        attributes: hashmap! {
                            "class".into() => "message-author".into(),
                        },
                        listeners: vec![],
                        children: vec![],
                        text: Some(std::borrow::Cow::Borrowed("[username not set]")),
                        key: None,
                    },
                    Element {
                        id: vec![0, 2, 2],
                        name: std::borrow::Cow::Borrowed("p"),
                        attributes: hashmap! {
                            "class".into() => "message-contents".into(),
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
