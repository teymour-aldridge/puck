use std::collections::HashMap;

use lunatic::{
    process::{self, this, Process},
    Mailbox,
};
use puck::ws::{
    message::Message,
    websocket::{NextMessageError, WebSocket},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    client::ClientMessage,
    dom::{
        element::{diff::changeset::instruction_serializer::InstructionSerializer, Element},
        event::{ClickEvent, InputEvent, SubmitEvent},
        listener::Listener,
    },
};

/// todo: documentation
pub trait Component<DATA, INPUT>
where
    INPUT: Serialize + DeserializeOwned,
{
    fn new(data: DATA, context: &Context<INPUT>) -> Self;

    fn update(&mut self, input: INPUT, context: &Context<INPUT>);

    fn render(&self) -> (Element, HashMap<String, Listener<INPUT>>);
}

#[derive(Debug)]
pub struct Context<INPUT>
where
    INPUT: serde::Serialize + serde::de::DeserializeOwned,
{
    proc_id: Process<INPUT>,
}

impl<INPUT> Context<INPUT>
where
    INPUT: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    pub fn process(&self) -> Process<INPUT> {
        self.proc_id.clone()
    }
}

/// Sets up the provided [Component] for communication over the WebSocket stream. The `Process`
/// returned can be used to send messages to the component.
// todo: simple code example
pub fn manage<COMPONENT, DATA, INPUT>(
    start_data: DATA,
    stream: WebSocket,
) -> Result<Process<INPUT>, lunatic::LunaticError>
where
    DATA: serde::Serialize + serde::de::DeserializeOwned,
    INPUT: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    COMPONENT: Component<DATA, INPUT>,
{
    process::spawn_with::<(DATA, WebSocket), INPUT>(
        (start_data, stream),
        |(start_data, stream), mailbox| {
            // spawn new process
            // todo: maybe have a supervisor
            let process = process::spawn_with(
                (start_data, stream.make_copy()),
                main_loop::<COMPONENT, DATA, INPUT>,
            )
            .unwrap();

            process.send(WsOrInput::WhoAmI(this(&mailbox)));

            process::spawn_with(
                (stream, process.clone()),
                |(mut websocket, process), _: Mailbox<()>| loop {
                    loop {
                        let msg = websocket.next();
                        if let Some(msg) = msg {
                            process.send(WsOrInput::Ws(msg))
                        }
                    }
                },
            )
            .unwrap();

            // forward all messages that this process receives to the child process
            loop {
                let msg = mailbox.receive();
                if let Ok(msg) = msg {
                    process.send(WsOrInput::Input(msg));
                }
            }
        },
    )
}

fn main_loop<COMPONENT, DATA, INPUT>(
    (start_data, stream): (DATA, WebSocket),
    mailbox: Mailbox<WsOrInput<INPUT>>,
) where
    DATA: serde::Serialize + serde::de::DeserializeOwned,
    INPUT: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    COMPONENT: Component<DATA, INPUT>,
{
    let context = Context {
        proc_id: match mailbox.receive().unwrap() {
            WsOrInput::WhoAmI(p) => p,
            _ => unreachable!(),
        },
    };

    let mut component = COMPONENT::new(start_data, &context);

    let (mut old_dom, mut old_listeners) = component.render();

    let instructions = old_dom.diff(None);

    let payload = serde_json::to_string(&InstructionSerializer(instructions)).unwrap();

    stream.send(Message::Text(payload)).unwrap();

    loop {
        let msg = mailbox.receive();
        match msg {
            Ok(WsOrInput::Ws(msg)) => {
                if let Ok(Message::Text(contents)) = msg {
                    // todo: this API is just plain messy
                    if let Ok(t) = serde_json::from_str::<ClientMessage>(&contents) {
                        if let Some(listener) = old_listeners.get(&t.listener) {
                            match listener {
                                Listener::Click { call } => {
                                    let input = (call)(ClickEvent);
                                    component.update(input, &context);
                                    perform_diff(
                                        &component,
                                        &mut old_dom,
                                        &stream,
                                        &mut old_listeners,
                                    );
                                }
                                Listener::Submit { call } => {
                                    let input = (call)(SubmitEvent);
                                    component.update(input, &context);
                                    perform_diff(
                                        &component,
                                        &mut old_dom,
                                        &stream,
                                        &mut old_listeners,
                                    );
                                }
                                Listener::Input { call } => {
                                    if let Some(payload) = t.payload {
                                        let input = (call)(InputEvent {
                                            value: payload.value,
                                        });
                                        component.update(input, &context);
                                        perform_diff(
                                            &component,
                                            &mut old_dom,
                                            &stream,
                                            &mut old_listeners,
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        {
                            continue;
                        }
                    };
                } else if let Err(e) = msg {
                    match e {
                        // todo: what should we do here?
                        NextMessageError::ClientError => {}
                        NextMessageError::ConnectionClosed => {
                            drop(component);
                            return;
                        }
                    }
                }
            }
            Ok(WsOrInput::Input(input)) => {
                component.update(input, &context);

                perform_diff(&component, &mut old_dom, &stream, &mut old_listeners);
            }
            Ok(_) | Err(_) => {}
        }
    }
}

fn perform_diff<COMPONENT, DATA, INPUT>(
    component: &COMPONENT,
    old_dom: &mut Element,
    stream: &WebSocket,
    old_listeners: &mut HashMap<String, Listener<INPUT>>,
) where
    DATA: serde::Serialize + serde::de::DeserializeOwned,
    INPUT: serde::Serialize + serde::de::DeserializeOwned,
    COMPONENT: Component<DATA, INPUT>,
{
    let (mut new_dom, mut new_listeners) = component.render();
    let instructions = old_dom.diff(Some(&new_dom));
    let _ = stream.send(Message::Text(
        serde_json::to_string(&InstructionSerializer(instructions)).unwrap(),
    ));
    std::mem::swap(old_dom, &mut new_dom);
    std::mem::swap(old_listeners, &mut new_listeners);
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "INPUT: serde::Serialize + for<'de2> serde::Deserialize<'de2>")]
enum WsOrInput<INPUT>
where
    INPUT: serde::Serialize + for<'de2> Deserialize<'de2> + std::fmt::Debug,
{
    Ws(Result<Message, NextMessageError>),
    Input(INPUT),
    // a process whose messages are sent to the Liveview component
    WhoAmI(Process<INPUT>),
}
