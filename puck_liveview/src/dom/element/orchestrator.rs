// todo: fix this

use std::{collections::HashMap, mem};

use lunatic::{
    channel::{Receiver, Sender},
    net::TcpStream,
    Process,
};

use puck::ws::{message::Message, websocket::WebSocket};
use serde::{Deserialize, Serialize};

use crate::{
    client::ClientMessage,
    dom::{
        event::{ClickEvent, InputEvent, SubmitEvent},
        listener::Listener,
    },
};

use super::{diff::changeset::instruction_serializer::JsFriendlyInstructionSerializer, Element};

pub trait Component<DATA, INPUT> {
    fn new(data: DATA) -> Self;

    fn update(&mut self, input: INPUT);

    fn render(&self) -> (Element, HashMap<String, Listener<INPUT>>);
}

pub fn manage<C: Component<DATA, INPUT>, DATA: 'static, INPUT>(
    stream: TcpStream,
    (oh_server_fill_me_with_messages, give_me_messages): (
        Sender<MessageWrapper<INPUT>>,
        Receiver<MessageWrapper<INPUT>>,
    ),
    start_data: DATA,
) where
    INPUT: Serialize + for<'de> Deserialize<'de>,
    DATA: Clone,
{
    Process::spawn_with(
        (stream.clone(), oh_server_fill_me_with_messages),
        |(stream, oh_server_fill_me_with_messages)| loop {
            let next = Message::next(stream.clone());
            match next {
                Ok(msg) => oh_server_fill_me_with_messages
                    .send(MessageWrapper::WebSocketMessage(msg))
                    .unwrap(),
                Err(_) => {
                    break;
                }
            }
        },
    )
    .detach();

    let mut component = C::new(start_data);

    let (mut old_dom, mut old_listeners) = component.render();

    let instructions = old_dom.diff(None);

    let payload = serde_json::to_string(&JsFriendlyInstructionSerializer(instructions)).unwrap();

    WebSocket::send_to_stream(stream.clone(), Message::Text(payload)).unwrap();

    while let Ok(input) = give_me_messages.receive() {
        match input {
            MessageWrapper::WebSocketMessage(msg) => {
                if let Message::Text(contents) = msg {
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

            MessageWrapper::WrappedMessageToPassOnToClient(input) => {
                component.update(input);

                let (mut new_dom, mut new_listeners) = component.render();

                let instructions = old_dom.diff(Some(&new_dom));

                WebSocket::send_to_stream(
                    stream.clone(),
                    Message::Text(
                        serde_json::to_string(&JsFriendlyInstructionSerializer(instructions))
                            .unwrap(),
                    ),
                )
                .unwrap();

                mem::swap(&mut old_dom, &mut new_dom);
                mem::swap(&mut old_listeners, &mut new_listeners);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
// todo: make this much tidier
pub enum MessageWrapper<WRAP> {
    WebSocketMessage(Message),
    WrappedMessageToPassOnToClient(WRAP),
}
