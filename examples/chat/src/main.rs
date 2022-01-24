use std::collections::{HashMap, HashSet};

use puck::core::router::match_url::{Match, Segment};
use puck::core::router::{Route, Router};
use puck::core::{Core, UsedStream};
use puck::lunatic::process::{spawn, Process};
use puck::lunatic::Mailbox;
use puck::ws::websocket::WebSocket;
use puck_liveview::component::Context;
use puck_liveview::dom::event::{ClickEvent, InputEvent};
use puck_liveview::dom::listener::ListenerRef;
use puck_liveview::html::id::IdGen;
use puck_liveview::prelude::*;
use puck_liveview::{component::Component, dom::listener::Listener};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// A single chat message.
struct ChatMessage {
    sender: UserId,
    username: Username,
    contents: String,
}

enum LiveviewComponent {
    Uninitialized {
        username: Username,
        orchestrator: Process<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>,
    },
    Initialized {
        username: Username,
        user_id: UserId,
        messages: Vec<ChatMessage>,
        currently_composing_message: String,
        orchestrator: Process<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>,
    },
}

impl LiveviewComponent {
    /// Returns `true` if the liveview component is [`Uninitialized`].
    ///
    /// [`Uninitialized`]: LiveviewComponent::Uninitialized
    fn is_uninitialized(&self) -> bool {
        matches!(self, Self::Uninitialized { .. })
    }

    /// Returns `true` if the liveview component is [`Initialized`].
    ///
    /// [`Initialized`]: LiveviewComponent::Initialized
    fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized { .. })
    }
}

impl Component<Process<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>, LiveviewInput>
    for LiveviewComponent
{
    fn new(
        orchestrator: Process<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>,
        _: &Context<LiveviewInput>,
    ) -> Self {
        Self::Uninitialized {
            username: Username(String::new()),
            orchestrator,
        }
    }

    fn update(&mut self, input: LiveviewInput, context: &Context<LiveviewInput>) {
        match input {
            LiveviewInput::UserRegistered(user_id, username) if self.is_uninitialized() => {
                let orchestrator = if let LiveviewComponent::Uninitialized {
                    username: _,
                    orchestrator,
                } = self
                {
                    orchestrator.clone()
                } else {
                    unreachable!()
                };

                *self = LiveviewComponent::Initialized {
                    username,
                    user_id,
                    messages: Vec::new(),
                    currently_composing_message: String::new(),
                    orchestrator,
                }
            }
            LiveviewInput::ReceiveMessage(message) if self.is_initialized() => match self {
                LiveviewComponent::Uninitialized { .. } => unreachable!(),
                LiveviewComponent::Initialized {
                    username: _,
                    user_id: _,
                    messages,
                    currently_composing_message: _,
                    orchestrator: _,
                } => {
                    messages.push(message);
                }
            },
            LiveviewInput::ReceiveMessage(_) if self.is_uninitialized() => {
                // do nothing
            }
            LiveviewInput::UpdatedUsername(new_username) if self.is_initialized() => match self {
                LiveviewComponent::Uninitialized { .. } => unreachable!(),
                LiveviewComponent::Initialized {
                    username,
                    user_id: _,
                    messages: _,
                    currently_composing_message: _,
                    orchestrator: _,
                } => *username = new_username,
            },
            LiveviewInput::BrowserRequest(BrowserRequest::InitializeUser)
                if self.is_uninitialized() =>
            {
                match self {
                    LiveviewComponent::Uninitialized {
                        username,
                        orchestrator,
                    } => {
                        let response = orchestrator
                            .request(CoordinatorInput::RegisterNewUser(
                                username.clone(),
                                context.process(),
                            ))
                            .unwrap();

                        if let LiveviewInput::UserRegistered(user_id, username) = response {
                            *self = LiveviewComponent::Initialized {
                                username,
                                user_id,
                                messages: vec![],
                                currently_composing_message: String::new(),
                                orchestrator: orchestrator.clone(),
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    LiveviewComponent::Initialized { .. } => unreachable!(),
                }
            }
            LiveviewInput::BrowserRequest(BrowserRequest::UpdateInitializeUsernameTextField(
                new_val,
            )) if self.is_uninitialized() => {
                if let LiveviewComponent::Uninitialized {
                    username,
                    orchestrator: _,
                } = self
                {
                    *username = Username(new_val)
                } else {
                    todo!()
                }
            }
            LiveviewInput::Closed => {}
            LiveviewInput::BrowserRequest(BrowserRequest::SendTextMessage)
                if self.is_initialized() =>
            {
                match self {
                    LiveviewComponent::Uninitialized { .. } => unreachable!(),
                    LiveviewComponent::Initialized {
                        username: _,
                        user_id,
                        messages: _,
                        currently_composing_message,
                        orchestrator,
                    } => {
                        let res = orchestrator
                            .request(CoordinatorInput::SendMessage(
                                *user_id,
                                currently_composing_message.clone(),
                            ))
                            .unwrap();
                        self.update(res, context);
                    }
                }
            }
            LiveviewInput::BrowserRequest(BrowserRequest::UpdateMessageContentsTextField(
                new_val,
            )) if self.is_initialized() => match self {
                LiveviewComponent::Uninitialized { .. } => unreachable!(),
                LiveviewComponent::Initialized {
                    username: _,
                    user_id: _,
                    messages: _,
                    currently_composing_message,
                    orchestrator: _,
                } => *currently_composing_message = new_val,
            },
            _ => {
                unreachable!()
            }
        }
    }

    fn render(
        &self,
    ) -> (
        puck_liveview::dom::element::Element,
        std::collections::HashMap<String, puck_liveview::dom::listener::Listener<LiveviewInput>>,
    ) {
        match self {
            LiveviewComponent::Uninitialized {
                username,
                orchestrator: _,
            } => (
                Div::new()
                    .wrap()
                    .child(
                        Div::new().wrap().child(
                            Input::new()
                                .attribute(Type::Text)
                                .attribute(Placeholder::new("username..."))
                                .attribute(Value::new(username.0.clone()))
                                .wrap()
                                .listener(ListenerRef::new("set_username", "input")),
                        ),
                    )
                    .child(
                        Div::new().wrap().child(
                            Input::new()
                                .attribute(Type::Submit)
                                .wrap()
                                .listener(ListenerRef::new("initialize_username", "click")),
                        ),
                    )
                    .into_element(&mut IdGen::new()),
                {
                    let mut listeners = HashMap::new();
                    listeners.insert(
                        "set_username".to_string(),
                        Listener::Input {
                            call: Box::new(|e: InputEvent| {
                                LiveviewInput::BrowserRequest(
                                    BrowserRequest::UpdateInitializeUsernameTextField(e.value),
                                )
                            }),
                        },
                    );
                    listeners.insert(
                        "initialize_username".to_string(),
                        Listener::Click {
                            call: Box::new(|_: ClickEvent| {
                                LiveviewInput::BrowserRequest(BrowserRequest::InitializeUser)
                            }),
                        },
                    );
                    listeners
                },
            ),
            LiveviewComponent::Initialized {
                username,
                user_id,
                messages,
                currently_composing_message,
                orchestrator: _,
            } => (
                Div::new()
                    .wrap()
                    .child(
                        Div::new()
                            .wrap()
                            .child(
                                P::new(format!("Hello {} (user_id: {})!", username.0, user_id.0))
                                    .wrap(),
                            )
                            .child(
                                Div::new().wrap().child(
                                    Input::new()
                                        .attribute(Value::new(currently_composing_message.clone()))
                                        .attribute(Type::Text)
                                        .attribute(Placeholder::new("send a new message..."))
                                        .wrap()
                                        .listener(ListenerRef::new(
                                            "update_message_contents",
                                            "input",
                                        )),
                                ),
                            )
                            .child(
                                Div::new().wrap().child(
                                    Input::new()
                                        .attribute(Type::Submit)
                                        .attribute(Value::new("send message"))
                                        .wrap()
                                        .listener(ListenerRef::new("submit_message", "click")),
                                ),
                            ),
                    )
                    .children(messages.iter().map(|message| {
                        Div::new()
                            .wrap()
                            .child(
                                H3::new(format!(
                                    "Message from {} (user id: {})",
                                    message.username.0, message.sender.0
                                ))
                                .wrap(),
                            )
                            .child(P::new(message.contents.clone()).wrap())
                    }))
                    .into_element(&mut IdGen::new()),
                {
                    let mut listeners = HashMap::new();

                    listeners.insert(
                        "update_message_contents".to_string(),
                        Listener::Input {
                            call: Box::new(|input| {
                                LiveviewInput::BrowserRequest(
                                    BrowserRequest::UpdateMessageContentsTextField(input.value),
                                )
                            }),
                        },
                    );

                    listeners.insert(
                        "submit_message".to_string(),
                        Listener::Click {
                            call: Box::new(|_click| {
                                LiveviewInput::BrowserRequest(BrowserRequest::SendTextMessage)
                            }),
                        },
                    );

                    listeners
                },
            ),
        }
    }
}

impl Drop for LiveviewComponent {
    fn drop(&mut self) {
        match self {
            LiveviewComponent::Uninitialized {
                username: _,
                orchestrator: _,
            } => {}
            LiveviewComponent::Initialized {
                username: _,
                user_id,
                messages: _,
                currently_composing_message: _,
                orchestrator,
            } => {
                let _ = orchestrator.request(CoordinatorInput::RemoveUser(*user_id));
            }
        }
    }
}

fn liveview(
    stream: WebSocket,
    state: Process<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>,
) -> UsedStream {
    puck_liveview::component::manage::<
        LiveviewComponent,
        Process<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>,
        LiveviewInput,
    >(state, stream)
    .unwrap();
    UsedStream::empty()
}

#[derive(Debug, Serialize, Deserialize)]
enum LiveviewInput {
    UserRegistered(UserId, Username),
    UpdatedUsername(Username),
    ReceiveMessage(ChatMessage),
    BrowserRequest(BrowserRequest),
    Closed,
}

#[derive(Debug, Serialize, Deserialize)]
/// Requests that the browser can send to the Liveview process.
enum BrowserRequest {
    UpdateInitializeUsernameTextField(String),
    InitializeUser,
    UpdateMessageContentsTextField(String),
    SendTextMessage,
}

#[derive(Serialize, Deserialize, Clone)]
enum CoordinatorInput {
    RegisterNewUser(Username, Process<LiveviewInput>),
    SetUsername(UserId, Username),
    SendMessage(UserId, String),
    RemoveUser(UserId),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, Hash, Eq, PartialEq)]
struct UserId(u64);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Username(String);

#[derive(Default)]
struct UserIdCreator {
    in_use: HashSet<UserId>,
    recycle: Vec<UserId>,
    head: UserId,
}

impl UserIdCreator {
    fn get_id(&mut self) -> UserId {
        let id = self.recycle.pop().unwrap_or_else(|| {
            let ret = self.head;
            self.head = UserId(self.head.0 + 1);
            ret
        });
        self.in_use.insert(id);
        id
    }

    fn release_id(&mut self, id: UserId) {
        if self.in_use.remove(&id) {
            self.recycle.push(id);
        }
    }
}

fn chat_server_coordinator(
    mailbox: Mailbox<puck::lunatic::Request<CoordinatorInput, LiveviewInput>>,
) {
    let mut users = HashMap::new();
    let mut id_gen = UserIdCreator::default();

    loop {
        if let Ok(msg) = mailbox.receive() {
            let data = msg.data().clone();
            match data {
                CoordinatorInput::RegisterNewUser(username, proc) => {
                    let id = id_gen.get_id();
                    users.insert(id, (username.clone(), proc));
                    msg.reply(LiveviewInput::UserRegistered(id, username))
                }
                CoordinatorInput::SetUsername(id, username) => {
                    users.entry(id).and_modify(|(u, _)| *u = username.clone());
                    msg.reply(LiveviewInput::UpdatedUsername(username.clone()))
                }
                CoordinatorInput::SendMessage(id, message) => {
                    let user_msg = ChatMessage {
                        sender: id,
                        username: users.get(&id).unwrap().0.clone(),
                        contents: message.clone(),
                    };

                    for (candidate, (_, proc)) in users.iter() {
                        if *candidate != id {
                            proc.send(LiveviewInput::ReceiveMessage(user_msg.clone()));
                        }
                    }

                    msg.reply(LiveviewInput::ReceiveMessage(user_msg))
                }
                CoordinatorInput::RemoveUser(id) => {
                    id_gen.release_id(id);
                    users.remove(&id);
                    msg.reply(LiveviewInput::Closed)
                }
            };
        }
    }
}

fn main() {
    let coordinator = spawn(chat_server_coordinator).unwrap();

    let router = Router::new()
        .route(Route::new(
            |req| Match::new().at(Segment::Static("js")).does_match(req.url()),
            |_, stream, _| stream.respond(puck_liveview::init::js()).unwrap(),
        ))
        .route(Route::new(
            |req| Match::new().at(Segment::Static("")).does_match(req.url()),
            |_, stream, _| stream.respond(puck_liveview::init::index()).unwrap(),
        ))
        .route(Route::new(
            |req| Match::new().at(Segment::Static("ws")).does_match(req.url()),
            |req, stream, state| {
                let websocket = match stream.upgrade(&req) {
                    Ok(w) => w,
                    Err(stream) => return stream,
                };
                liveview(websocket, state)
            },
        ));

    Core::bind("localhost:8081", coordinator)
        .expect("failed to serve")
        .serve_router(router);
}
