//! A simple chat server.
//!
//! # Architecture.
//! ```
//! ┌─────────────────────────┐
//! │  Co-ordinating server   │◀──────┬───────────┐
//! └─────────────────────────┘       │  Talk to  │
//!                                   │each other │
//!                                   └───────────┤
//!                                               │
//!                                               │
//!                                               │
//!                                               │
//!                                               ▼
//!                                     ┌───────────────────┐
//!                                     │  Client manager   │
//!                                     └───────────────────┘
//!                                               ▲
//!                                               │
//!                                               ├───────────┐
//!                                               │ WebSocket │
//!                                               │connection │
//!                                               ├───────────┘
//!                                               │
//!                                               │
//!                                               ▼
//!                                       ┌──────────────┐
//!                                       │   Browser    │
//!                                       └──────────────┘
//! ```

use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use lunatic::{
    channel::{bounded, unbounded, Receiver, Sender},
    net::TcpStream,
};
use maplit::hashmap;
use puck::{serve, Request};
use puck_liveview::{
    dom::{
        element::{
            orchestrator::{manage, Component, MessageWrapper},
            Element,
        },
        event::{ClickEvent, InputEvent},
        listener::{Listener, ListenerRef},
    },
    init::{index, js},
    prelude::*,
};
use serde::{Deserialize, Serialize};

fn main() {
    serve::<App, &str>("127.0.0.1:5052").expect("server error");
}

#[puck::handler(
    handle(at = "/ws", call = "liveview", web_socket = true, send = "chat"),
    handle(at = "/", call = "index"),
    handle(at = "/js", call = "js"),
    channel(
        name = "chat",
        message_type = "SubscribeMsg",
        supervisor = "chat_server"
    )
)]
struct App;

#[derive(Debug, Clone)]
struct UserChatData {
    messages: Vec<Msg>,
    username: String,
    text_field_contents: String,
    process_id: u32,
}

impl Default for UserChatData {
    fn default() -> Self {
        unimplemented!()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Msg {
    from: String,
    contents: String,
    sent_at: NaiveDateTime,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum InputMsg {
    NewMsg(Msg),
    TextInput(String),
    SendMsg,
}

fn liveview(_: Request, stream: TcpStream, sender: Sender<SubscribeMsg>) {
    let (send_username, receive_username) = bounded(1);
    sender.send(SubscribeMsg::Joined(send_username)).unwrap();
    let id = receive_username.receive().unwrap();

    let data = UserChatData {
        messages: vec![],
        username: "[username not set]".to_string(),
        text_field_contents: String::new(),
        process_id: id,
    };

    let (oh_server_fill_me_with_messages, give_me_messages) = unbounded();

    sender
        .send(SubscribeMsg::Subscribe(
            id,
            oh_server_fill_me_with_messages.clone(),
        ))
        .unwrap();

    manage::<Root, (UserChatData, Sender<SubscribeMsg>), InputMsg>(
        stream,
        (oh_server_fill_me_with_messages, give_me_messages),
        (data, sender),
    );
}

pub struct Root {
    data: UserChatData,
    sender: Sender<SubscribeMsg>,
}

impl Component<(UserChatData, Sender<SubscribeMsg>), InputMsg> for Root {
    fn new(data: (UserChatData, Sender<SubscribeMsg>)) -> Self {
        let (data, sender) = data;
        Self { data, sender }
    }

    fn update(&mut self, input: InputMsg) {
        match input {
            InputMsg::NewMsg(msg) => self.data.messages.push(msg),
            InputMsg::TextInput(contents) => {
                self.data.text_field_contents = contents;
            }
            InputMsg::SendMsg => {
                self.sender
                    .send(SubscribeMsg::SendMsg {
                        sending_process_id: self.data.process_id,
                        msg: Msg {
                            from: self.data.username.clone(),
                            contents: self.data.text_field_contents.clone(),
                            sent_at: Utc::now().naive_utc(),
                        },
                    })
                    .unwrap();
                self.data.text_field_contents = "".to_string();
            }
        }
    }

    fn render(&self) -> (Element, HashMap<String, Listener<InputMsg>>) {
        // as you can see, the API needs some work

        (
            Div::new()
                .wrap()
                .child(
                    Div::new().wrap().child(
                        Input::new()
                            .attribute(Type::Text)
                            .wrap()
                            .listener(ListenerRef::new("msg-input", "input")),
                    ),
                )
                .child(
                    Div::new().wrap().child(
                        Input::new()
                            .attribute(Type::Submit)
                            .wrap()
                            .listener(ListenerRef::new("msg-submit", "click")),
                    ),
                )
                .children(self.data.messages.iter().map(|message| {
                    Div::new()
                        .attribute(Class::from("message-container"))
                        .wrap()
                        .child(
                            P::with_text(message.sent_at.format("%Y-%m-%d %H:%M:%S").to_string())
                                .wrap(),
                        )
                        .child(P::with_text(format!("Sent by: {}", message.from.clone())).wrap())
                        .child(P::with_text(message.contents.clone()).wrap())
                }))
                .into_element(vec![0]),
            hashmap! {
                "msg-input".to_string() => Listener::Input {
                    call: Box::new(|e: InputEvent| {
                        InputMsg::TextInput(
                            e.value
                        )
                    })
                },
                "msg-submit".to_string() => Listener::Click {
                    call: Box::new(|_: ClickEvent| {
                        InputMsg::SendMsg
                    })
                }
            },
        )
    }
}

#[derive(Serialize, Deserialize)]
pub enum SubscribeMsg {
    Joined(Sender<u32>),
    Subscribe(u32, Sender<MessageWrapper<InputMsg>>),
    SendMsg { sending_process_id: u32, msg: Msg },
}

fn chat_server((_, receiver): (Sender<SubscribeMsg>, Receiver<SubscribeMsg>)) {
    let mut id_counter = 0;
    let mut id_to_channel_mapper = HashMap::new();

    while let Ok(msg) = receiver.receive() {
        match msg {
            SubscribeMsg::Joined(sender) => {
                sender.send(id_counter).unwrap();
                id_counter += 1;
            }
            SubscribeMsg::Subscribe(id, stream) => {
                id_to_channel_mapper.insert(id, stream);
            }
            SubscribeMsg::SendMsg {
                sending_process_id,
                msg,
            } => {
                for (id, sender) in id_to_channel_mapper.iter() {
                    if sending_process_id != *id {
                        sender
                            .send(MessageWrapper::WrappedMessageToPassOnToClient(
                                InputMsg::NewMsg(msg.clone()),
                            ))
                            .expect("failed to send message");
                    }
                }
            }
        }
    }
}
