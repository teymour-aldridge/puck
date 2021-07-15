//! The root web server application.

use std::fmt;

use lunatic::net::TcpStream;
use url::Url;

use crate::{
    body::Body,
    response::encoder::Encoder,
    ws::{perform_upgrade, should_upgrade, websocket::WebSocket},
    Request, Response,
};

#[allow(missing_docs)]

pub enum Handler<STATE> {
    Ws(fn(WebSocket<TcpStream>, STATE)),
    Http(fn(Request, STATE) -> Response),
}

impl<STATE> fmt::Debug for Handler<STATE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Handler::Ws(_) => write!(f, "Ws"),
            Handler::Http(_) => write!(f, "Http"),
        }
    }
}

#[allow(missing_docs)]
pub struct Route<STATE> {
    handler: Handler<STATE>,
    matcher: fn(&Url) -> bool,
}

#[macro_export]
/// Create a function matching the specified path.
macro_rules! at {
    ($($string:expr) => +) => {
        |url| -> bool {
            url.path_segments().map(|mut split| {
                $(
                    if let Some(next) = split.next() {
                        if next != $string {
                            return false;
                        };
                    } else {
                        return false;
                    }
                )*
                true
            }).unwrap_or(false)
        }
    };
}

impl<STATE> Route<STATE> {
    /// Create a new route.
    pub fn new(matcher: fn(&Url) -> bool, handler: Handler<STATE>) -> Self {
        Self { handler, matcher }
    }

    fn matches(&self, url: &Url) -> bool {
        (self.matcher)(url)
    }
}

impl<STATE> fmt::Debug for Route<STATE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("handler", &self.handler)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
#[allow(missing_docs)]
pub struct App<STATE> {
    routes: Vec<Route<STATE>>,
    state: Option<STATE>,
}

impl<STATE> Default for App<STATE> {
    fn default() -> Self {
        Self {
            routes: Default::default(),
            state: Default::default(),
        }
    }
}

impl<STATE> App<STATE> {
    /// Create a new route.
    pub fn new() -> Self {
        Default::default()
    }

    /// Attach the required state to this application.
    pub fn state(mut self, state: STATE) -> Self {
        self.state = Some(state);
        self
    }

    /// Attach a new route to this application.
    pub fn route(mut self, route: Route<STATE>) -> Self {
        self.routes.push(route);
        self
    }

    /// Process a single request.
    pub fn process_request(self, stream: TcpStream) {
        let request = if let Ok(Some(request)) = Request::parse(stream.clone()) {
            request
        } else {
            // todo: allow custom error messages
            Encoder::new(
                Response::build()
                    .status(400, "bad request")
                    .body(Body::from_string("<h1>Error 400: bad request</h1>"))
                    .build(),
            )
            .write_tcp_stream(stream)
            .unwrap();
            return;
        };

        let url = request.url();

        for route in self.routes {
            if route.matches(url) {
                match route.handler {
                    Handler::Ws(handler) => {
                        if !should_upgrade(&request) {
                            Encoder::new(
                                Response::build()
                                    .status(400, "bad request")
                                    .body(Body::from_string("<h1>Error 400: bad request</h1>"))
                                    .build(),
                            )
                            .write_tcp_stream(stream)
                            .unwrap();
                            return;
                        }
                        if !perform_upgrade(&request, stream.clone()) {
                            return;
                        }
                        let ws = WebSocket::new(stream);
                        (handler)(
                            ws,
                            self.state.expect("internal error – state not available"),
                        )
                    }
                    Handler::Http(handler) => {
                        let response = handler(
                            request,
                            self.state.expect("internal error – state not available"),
                        );

                        Encoder::new(response).write_tcp_stream(stream).unwrap();
                    }
                }
                return;
            }
        }

        Encoder::new(
            Response::build()
                .status(404, "not found")
                .body(Body::from_string("<h1>Error 404: not found</h1>"))
                .build(),
        )
        .write_tcp_stream(stream)
        .unwrap();
    }
}

#[macro_export]
/// Run the app on the provided address.
macro_rules! run_app {
    ($make_app:expr, $addr:expr, $state:expr) => {{
        let listener = ::lunatic::net::TcpListener::bind($addr).expect(
            "failed to start the application on the provided address; are you sure it's correct",
        );

        loop {
            let stream = if let Ok(stream) = listener.accept() {
                stream
            } else {
                continue;
            };

            ::lunatic::process::Process::spawn_with((stream, $state.clone()), |(stream, state)| {
                let app: $crate::app::App<_> = $make_app.state(state);
                app.process_request(stream);
            })
            .detach();
        }
    }};
}
