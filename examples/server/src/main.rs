use std::collections::HashMap;

use puck::{
    request::{Body, Method, HTML},
    Request, Response,
};

fn home(req: Request) -> Response {
    Response {
        headers: {
            let mut res = HashMap::new();
            res.insert("Content-Type".to_string(), HTML.to_string());
            res
        },
        body: Body::from_string(req.url.to_string()),
        status: 200,
        reason: "success".to_string(),
        method: Method::Get,
    }
}

fn hello(_: Request) -> Response {
    Response {
        headers: {
            let mut res = HashMap::new();
            res.insert("Content-Type".to_string(), HTML.to_string());
            res
        },
        body: Body::from_string("<h1>Hello!</h1>".to_string()),
        status: 200,
        reason: "success".to_string(),
        method: Method::Get,
    }
}

#[puck::handler(
    handle(at = "/", function = "home"),
    handle(at = "/hello", function = "hello")
)]
pub struct App;

fn main() {
    puck::serve::<App>("127.0.0.1:5050").unwrap()
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};

    use puck::lunatic::{self};

    #[test]
    fn sanity_checks() {
        fn proc(_: ()) {
            super::main()
        }
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
}
