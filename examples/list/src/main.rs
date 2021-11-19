use lunatic::{process, Mailbox};
use malvolio::prelude::*;
use puck::{body::Body, core::Core, Response};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let proc = process::spawn(list).expect("failed to launch process");

    Core::bind("localhost:8080", proc)
        .unwrap()
        .for_each(|mut request, stream, state| {
            // todo: iron this API out
            if request.url().path() == "/submit" {
                match request.method() {
                    puck::request::Method::Get => stream
                        .respond(
                            Response::build()
                                .headers(vec![(
                                    "Content-Type".to_string(),
                                    "text/html".to_string(),
                                )])
                                .body(Body::from_string(
                                    html().head(head().child(title("Submit a message"))).body(
                                        body().child(
                                            form()
                                                .attribute(malvolio::prelude::Method::Post)
                                                .child(input().attribute(Name::new("message")))
                                                .child(input().attribute(Type::Submit)),
                                        ),
                                    ),
                                ))
                                .build(),
                        )
                        .unwrap(),
                    puck::request::Method::Post => {
                        let res = request.take_body().into_string().unwrap();

                        if res.starts_with("message=") {
                            // beware of how utf-8 works if you copy this
                            let seg = res.split_at("message=".len()).1;

                            match state.request(Msg::Add(seg.to_string())).unwrap() {
                                Reply::Items(_) => unreachable!(),
                                Reply::Added => stream
                                    .respond(
                                        Response::build()
                                            .headers(vec![(
                                                "Content-Type".to_string(),
                                                "text/html".to_string(),
                                            )])
                                            .body(Body::from_string(
                                                html()
                                                    .head(head().child(title("Submit a message")))
                                                    .body(body().child(h1("Added that item"))),
                                            ))
                                            .build(),
                                    )
                                    .unwrap(),
                            }
                        } else {
                            stream.respond(puck::err_400()).unwrap()
                        }
                    }
                    _ => stream.respond(puck::err_400()).unwrap(),
                }
            } else if request.url().path().starts_with("/read/") {
                let segment = request.url().path().split_at("/read/".len()).1;
                if let Ok(n) = segment.parse::<usize>() {
                    let res = state.request(Msg::LastN(n)).unwrap();
                    let items = match res {
                        Reply::Items(items) => items,
                        Reply::Added => unreachable!(),
                    };
                    stream
                        .respond(
                            puck::Response::build()
                                .headers(vec![(
                                    "Content-Type".to_string(),
                                    "text/html".to_string(),
                                )])
                                .body(Body::from_string(
                                    html().head(head().child(title("Message list"))).body(
                                        body().child(h1("Message list")).map(|body| {
                                            if items.is_empty() {
                                                body.child(p().text("There are no messages yet."))
                                            } else {
                                                body.children(items.into_iter().map(|item| {
                                                    p().text(format!("Item: {}", item))
                                                }))
                                            }
                                        }),
                                    ),
                                ))
                                .build(),
                        )
                        .unwrap()
                } else {
                    stream.respond(puck::err_404()).unwrap()
                }
            } else {
                stream.respond(puck::err_404()).unwrap()
            }
        });
}

#[derive(serde::Serialize, serde::Deserialize)]
enum Msg {
    Add(String),
    AllItems,
    LastN(usize),
}

#[derive(serde::Serialize, serde::Deserialize)]
enum Reply {
    Items(Vec<String>),
    Added,
}

fn list(mailbox: Mailbox<lunatic::Request<Msg, Reply>>) {
    let mut items: Vec<String> = vec![];

    loop {
        let req = match mailbox.receive() {
            Ok(req) => req,
            Err(_) => {
                continue;
            }
        };

        match req.data() {
            Msg::Add(string) => {
                items.push(string.to_string());
                req.reply(Reply::Added);
            }
            Msg::AllItems => req.reply(Reply::Items(items.clone())),
            Msg::LastN(n) => {
                if items.len() < *n {
                    req.reply(Reply::Items(items.clone()))
                } else {
                    req.reply(Reply::Items(items.get(0..).unwrap().to_vec()))
                }
            }
        }
    }
}
