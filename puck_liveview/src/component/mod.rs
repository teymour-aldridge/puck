use std::collections::HashMap;

use lunatic::{
    process::{self, Process},
    Mailbox,
};
use puck::ws::{message::Message, websocket::WebSocket};

use crate::{
    client::ClientMessage,
    dom::{
        element::{diff::changeset::instruction_serializer::InstructionSerializer, Element},
        event::{ClickEvent, InputEvent, SubmitEvent},
        listener::Listener,
    },
};

/// todo: documentation
pub trait Component<DATA, INPUT> {
    fn new(data: DATA) -> Self;

    fn update(&mut self, input: INPUT);

    fn render(&self) -> (Element, HashMap<String, Listener<INPUT>>);
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
    INPUT: serde::Serialize + serde::de::DeserializeOwned,
    COMPONENT: Component<DATA, INPUT>,
{
    process::spawn_with::<(DATA, WebSocket), INPUT>(
        (start_data, stream),
        |(start_data, stream), mailbox| {
            // spawn new process
            // todo: maybe have a supervisor
            let process =
                process::spawn_with((start_data, stream), main_loop::<COMPONENT, DATA, INPUT>)
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
    INPUT: serde::Serialize + serde::de::DeserializeOwned,
    COMPONENT: Component<DATA, INPUT>,
{
    let mut component = COMPONENT::new(start_data);

    let (mut old_dom, mut old_listeners) = component.render();

    let instructions = old_dom.diff(None);

    let payload = serde_json::to_string(&InstructionSerializer(instructions)).unwrap();

    stream.send(Message::Text(payload)).unwrap();

    loop {
        let msg = mailbox.receive();
        match msg {
            Ok(WsOrInput::Ws(msg)) => {
                if let Message::Text(contents) = msg {
                    // todo: this API is just plain messy
                    if let Ok(t) = serde_json::from_str::<ClientMessage>(&contents) {
                        if let Some(listener) = old_listeners.get(&t.listener) {
                            match listener {
                                Listener::Click { call } => {
                                    let input = (call)(ClickEvent);
                                    component.update(input);
                                }
                                Listener::Submit { call } => {
                                    let input = (call)(SubmitEvent);
                                    component.update(input);
                                }
                                Listener::Input { call } => {
                                    if let Some(payload) = t.payload {
                                        let input = (call)(InputEvent {
                                            value: payload.value,
                                        });
                                        component.update(input);
                                    }
                                }
                            }
                        }
                    } else {
                        {
                            continue;
                        }
                    };
                }
            }
            Ok(WsOrInput::Input(input)) => {
                component.update(input);

                let (mut new_dom, mut new_listeners) = component.render();

                let instructions = old_dom.diff(Some(&new_dom));

                let _ = stream.send(Message::Text(
                    serde_json::to_string(&InstructionSerializer(instructions)).unwrap(),
                ));

                std::mem::swap(&mut old_dom, &mut new_dom);
                std::mem::swap(&mut old_listeners, &mut new_listeners);
            }
            Err(_) => {}
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
enum WsOrInput<INPUT> {
    Ws(Message),
    Input(INPUT),
}
