use malvolio::prelude::*;

use std::collections::HashMap;

use crate::dom::element::Element;
use crate::dom::listener::ListenerRef;
use crate::prelude::*;

use crate::html::id::IdGen;

#[test]
fn diffing_regression_2021_06_06() {
    let mut before = H1::new("")
        .raw_attribute("¡", "")
        .wrap()
        .into_element(&mut IdGen::new());
    let expected_after = H1::new("").wrap().into_element(&mut IdGen::new());

    let diff_before = before.clone();

    let cs = diff_before.diff(Some(&expected_after));

    cs.apply(&mut before);

    assert_eq!(before, expected_after);
}

#[test]
fn diffing_regression_2022_01_23() {
    let mut before = Element {
        id: 0,
        name: "div".into(),
        attributes: HashMap::new(),
        listeners: vec![],
        children: vec![
            Element {
                id: 1,
                name: "div".into(),
                attributes: HashMap::new(),
                listeners: vec![],
                children: vec![Element {
                    id: 2,
                    name: "input".into(),
                    attributes: {
                        let mut res = HashMap::new();
                        res.insert("type".into(), "text".into());
                        res.insert("value".into(), "name".into());
                        res.insert("placeholder".into(), "username...".into());
                        res
                    },
                    listeners: vec![ListenerRef {
                        listener_name: "set_username".into(),
                        js_event: "input".into(),
                    }],
                    children: vec![],
                    text: None,
                    key: None,
                }],
                text: None,
                key: None,
            },
            Element {
                id: 3,
                name: "div".into(),
                attributes: HashMap::new(),
                listeners: vec![],
                children: vec![Element {
                    id: 4,
                    name: "input".into(),
                    attributes: {
                        let mut res = HashMap::new();
                        res.insert("type".into(), "submit".into());
                        res
                    },
                    listeners: vec![ListenerRef {
                        listener_name: "initialize_username".into(),
                        js_event: "click".into(),
                    }],
                    children: vec![],
                    text: None,
                    key: None,
                }],
                text: None,
                key: None,
            },
        ],
        text: None,
        key: None,
    };

    let after = Element {
        id: 0,
        name: "div".into(),
        attributes: HashMap::new(),
        listeners: vec![],
        children: vec![Element {
            id: 1,
            name: "div".into(),
            attributes: HashMap::new(),
            listeners: vec![],
            children: vec![
                Element {
                    id: 2,
                    name: "p".into(),
                    attributes: HashMap::new(),
                    listeners: vec![],
                    children: vec![],
                    text: Some("Hello teymour (user_id: 0)!".into()),
                    key: None,
                },
                Element {
                    id: 3,
                    name: "div".into(),
                    attributes: HashMap::new(),
                    listeners: vec![],
                    children: vec![Element {
                        id: 4,
                        name: "input".into(),
                        attributes: {
                            let mut res = HashMap::new();
                            res.insert("placeholder".into(), "send a new message...".into());
                            res.insert("type".into(), "text".into());
                            res.insert("value".into(), "".into());
                            res
                        },
                        listeners: vec![ListenerRef {
                            listener_name: "update_message_contents".into(),
                            js_event: "input".into(),
                        }],
                        children: vec![],
                        text: None,
                        key: None,
                    }],
                    text: None,
                    key: None,
                },
                Element {
                    id: 5,
                    name: "div".into(),
                    attributes: HashMap::new(),
                    listeners: vec![],
                    children: vec![Element {
                        id: 6,
                        name: "input".into(),
                        attributes: {
                            let mut res = HashMap::new();
                            res.insert("type".into(), "submit".into());
                            res.insert("value".into(), "send message".into());
                            res
                        },
                        listeners: vec![ListenerRef {
                            listener_name: "submit_message".into(),
                            js_event: "click".into(),
                        }],
                        children: vec![],
                        text: None,
                        key: None,
                    }],
                    text: None,
                    key: None,
                },
            ],
            text: None,
            key: None,
        }],
        text: None,
        key: None,
    };

    let before2 = before.clone();
    let cs = before2.diff(Some(&after));

    cs.apply(&mut before);

    assert_eq!(before, after);
}

#[test]
fn simple_listener_test() {
    let mut before = Element {
        id: 0,
        listeners: vec![
            ListenerRef::new("one", "two"),
            ListenerRef::new("three", "four"),
        ],
        ..Default::default()
    };
    let after = Element {
        id: 0,
        listeners: vec![ListenerRef::new("one", "two")],
        ..Default::default()
    };
    let before2 = before.clone();
    let cs = before2.diff(Some(&after));

    cs.apply(&mut before);
    assert_eq!(before, after);
}
