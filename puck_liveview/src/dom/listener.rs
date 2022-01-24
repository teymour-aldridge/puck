use std::fmt::Debug;

use super::event::{ClickEvent, InputEvent, SubmitEvent};

/// A DOM listener.

pub enum Listener<INPUT> {
    Click {
        call: Box<dyn Fn(ClickEvent) -> INPUT>,
    },
    Submit {
        call: Box<dyn Fn(SubmitEvent) -> INPUT>,
    },
    Input {
        call: Box<dyn Fn(InputEvent) -> INPUT>,
    },
}

impl<INPUT> Debug for Listener<INPUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listener").finish()
    }
}

impl<INPUT> From<Box<dyn Fn(ClickEvent) -> INPUT>> for Listener<INPUT> {
    fn from(call: Box<dyn Fn(ClickEvent) -> INPUT>) -> Self {
        Self::Click { call }
    }
}

impl<INPUT> From<Box<dyn Fn(SubmitEvent) -> INPUT>> for Listener<INPUT> {
    fn from(call: Box<dyn Fn(SubmitEvent) -> INPUT>) -> Self {
        Self::Submit { call }
    }
}

impl<INPUT> From<Box<dyn Fn(InputEvent) -> INPUT>> for Listener<INPUT> {
    fn from(call: Box<dyn Fn(InputEvent) -> INPUT>) -> Self {
        Self::Input { call }
    }
}

impl<INPUT> Listener<INPUT> {
    #[inline]
    pub fn js_event(&self) -> &'static str {
        match self {
            Listener::Click { call: _ } => "click",
            Listener::Submit { call: _ } => "submit",
            Listener::Input { call: _ } => "input",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerRef {
    pub(crate) listener_name: String,
    pub(crate) js_event: String,
}

impl ListenerRef {
    pub fn new(listener_name: impl ToString, js_event: impl ToString) -> Self {
        Self {
            listener_name: listener_name.to_string(),
            js_event: js_event.to_string(),
        }
    }
    /// Get a reference to the listener ref's listener name.
    pub fn listener_name(&self) -> &str {
        self.listener_name.as_str()
    }

    /// Get a reference to the listener ref's js event.
    pub fn js_event(&self) -> &str {
        self.js_event.as_str()
    }
}
