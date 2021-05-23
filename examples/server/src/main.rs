use puck::{
    lunatic::channel::{Receiver, Sender},
    request::{Body, Method, HTML, PLAIN},
    Request, Response,
};
use serde::{Deserialize, Serialize};

fn home(_: Request) -> Response {
    Response::build()
        .header("Content-Type", HTML)
        .body(Body::from_string("Hello World!"))
        .status(200, "success")
        .build()
}

fn hello(req: Request) -> Response {
    let name = req.url.path().split('/').last().unwrap();

    Response::build()
        .header("Content-Type", HTML)
        .body(Body::from_string(format!("<h1>Hello {}!</h1>", name)))
        .status(200, "success")
        .method(Method::Get)
        .build()
}

#[derive(Serialize, Deserialize, Debug)]
enum Msg {
    Send(String),
}

fn submit_info(req: Request, sender: Sender<Msg>) -> Response {
    sender
        .send(Msg::Send(
            req.url.path().split('/').last().unwrap().to_string(),
        ))
        .unwrap();
    Response::build()
        .header("Content-Type", PLAIN)
        .body(Body::from_string("Submitted".to_string()))
        .status(200, "success")
        .method(Method::Get)
        .build()
}

fn read_info(_: Request, reader: Receiver<Msg>) -> Response {
    Response::build()
        .header("Content-Type", PLAIN)
        .body(Body::from_string(match reader.receive().unwrap() {
            Msg::Send(msg) => msg,
        }))
        .status(200, "success")
        .method(Method::Get)
        .build()
}

fn echo_echo((_send, receive): (Sender<Msg>, Receiver<Msg>)) {
    while let Ok(t) = receive.receive() {
        println!("{:#?}", t);
    }
}

#[puck::handler(
    handle(at = "/", call = "home"),
    handle(at = "/hello/<string>", call = "hello"),
    handle(at = "/submit/<string>", call = "submit_info", send = "echo"),
    handle(at = "/read", call = "read_info", receive = "echo"),
    channel(name = "echo", message_type = "Msg", supervisor = "echo_echo")
)]
pub struct App;

fn main() {
    puck::serve::<App>("127.0.0.1:5050").unwrap()
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};

    use puck::lunatic::{self};

    fn proc(_: ()) {
        super::main()
    }

    #[test]
    fn sanity_checks() {
        lunatic::Process::spawn_with((), proc).detach();
        fn inner(_: ()) {
            let mut stream = lunatic::net::TcpStream::connect("127.0.0.1:5050").unwrap();
            write!(stream, "GET / HTTP/1.1\r\nHost: localhost:5050\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n").unwrap();
            let mut string = String::new();
            stream.read_to_string(&mut string).unwrap();
            assert_eq!(
                string,
                "HTTP/1.1 200 success\r\nContent-Type: text/html;charset=utf-8\r\n\r\nhttp://localhost:5050/"
                .to_string()
            );
        }
        lunatic::Process::spawn_with((), inner).detach();
    }
    #[test]
    fn test_channels() {
        lunatic::Process::spawn_with((), proc).detach();
        fn inner(_: ()) {
            let mut stream = lunatic::net::TcpStream::connect("127.0.0.1:5050").unwrap();
            write!(
                stream,
                "GET /submit/hello HTTP/1.1\r\nHost: localhost:5050\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n"
            )
            .unwrap();
            let mut string = String::new();
            stream.read_to_string(&mut string).unwrap();
            assert_eq!(
                string,
                "HTTP/1.1 200 \r\nContent-Type: text/plain;charset=utf-8\r\n\r\nSubmitted"
            );

            let mut stream = lunatic::net::TcpStream::connect("127.0.0.1:5050").unwrap();
            write!(
                stream,
                "GET /read HTTP/1.1\r\nHost: localhost:5050\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n"
            )
            .unwrap();
            let mut string = String::new();
            stream.read_to_string(&mut string).unwrap();
            assert_eq!(
                string,
                "HTTP/1.1 200 success\r\nContent-Type: text/html;charset=utf-8\r\n\r\nhello"
            );
        }
        lunatic::Process::spawn_with((), inner).detach();
    }
}
