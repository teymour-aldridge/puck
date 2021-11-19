//! A router.
use std::{fmt, mem};

use lunatic::{net::TcpListener, process, Mailbox};
use serde::{de::DeserializeOwned, Serialize};

use crate::Request;

use super::{Stream, UsedStream};

#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct Route<STATE> {
    matcher: fn(&Request) -> bool,
    handler: fn(Request, Stream, STATE) -> UsedStream,
}

impl<STATE> fmt::Debug for Route<STATE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route").finish()
    }
}

impl<STATE> Route<STATE> {
    /// Constructs a new `Route`.
    ///
    /// A substantially nicer API will come.
    pub fn new(
        matcher: fn(&Request) -> bool,
        handler: fn(Request, Stream, STATE) -> UsedStream,
    ) -> Self {
        Self { matcher, handler }
    }
}

/// A [Router] provides an easy way to match different types of HTTP request and handle them
/// differently.
#[derive(Debug, Clone)]
pub struct Router<STATE> {
    routes: Vec<Route<STATE>>,
}

impl<STATE: Serialize + DeserializeOwned + Clone> Router<STATE> {
    /// Constructs a new [Router].
    pub fn new(routes: impl IntoIterator<Item = Route<STATE>>) -> Self {
        Self {
            routes: routes.into_iter().collect::<Vec<_>>(),
        }
    }

    fn as_ints(&self) -> Vec<(usize, usize)> {
        self.routes
            .iter()
            .map(|route| {
                (
                    route.matcher as *const () as usize,
                    route.handler as *const () as usize,
                )
            })
            .collect()
    }

    /// Reconstructs the router from `Router::as_ints`. Panics if the data is not in a valid form.
    fn from_ints(ints: Vec<(usize, usize)>) -> Self {
        let routes = ints
            .iter()
            .map(|(matcher, handler)| Route {
                matcher: {
                    unsafe {
                        let pointer = *matcher as *const ();
                        mem::transmute::<*const (), fn(&Request) -> bool>(pointer)
                    }
                },
                handler: {
                    unsafe {
                        let pointer = *handler as *const ();
                        mem::transmute::<*const (), fn(Request, Stream, STATE) -> UsedStream>(
                            pointer,
                        )
                    }
                },
            })
            .collect::<Vec<_>>();
        Self { routes }
    }

    /// Runs the router forever on the provided port.
    pub fn run(self, listener: TcpListener, state: STATE) {
        loop {
            let stream = if let Ok((stream, _addr)) = listener.accept() {
                stream
            } else {
                continue;
            };

            let _ = process::spawn_with(
                (self.as_ints(), stream, state.clone()),
                |(ints, stream, state), _: Mailbox<()>| {
                    if let Ok(Some(req)) = Request::parse(stream.clone()) {
                        let router = Self::from_ints(ints);
                        router.respond(req, Stream::new(stream, false), state);
                    }
                },
            );
        }
    }

    fn respond(&self, req: Request, stream: Stream, state: STATE) {
        for route in &self.routes {
            if (route.matcher)(&req) {
                (route.handler)(req, stream, state);
                return;
            }
        }
    }
}