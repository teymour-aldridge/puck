//!

use std::{io, mem};

use lunatic::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    process, Mailbox,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    response::encoder::Encoder,
    ws::{self, websocket::WebSocket},
    Request, Response,
};

use self::router::Router;

pub mod router;

///
#[derive(Debug)]
pub struct Core<STATE> {
    state: STATE,
    listener: TcpListener,
}

impl<STATE> Core<STATE>
where
    STATE: Clone + Serialize + DeserializeOwned,
{
    /// Bind
    pub fn bind(addr: impl ToSocketAddrs, state: STATE) -> Result<Self, io::Error> {
        Ok(Self {
            state,
            listener: TcpListener::bind(addr)?,
        })
    }

    /// Serves the current router, forever, on the bound address.
    pub fn serve_router(self, router: Router<STATE>) {
        let ints = router.as_ints();

        loop {
            if let Ok((stream, _)) = self.listener.accept() {
                let _ = process::spawn_with(
                    (stream, ints.clone(), self.state.clone()),
                    |(stream, ints, state), _: Mailbox<()>| {
                        let router = Router::<STATE>::from_ints(ints);

                        let req = if let Some(req) = Request::parse(stream.clone()).unwrap() {
                            req
                        } else {
                            let stream = Stream::new(stream, false);
                            // can't do much if this fails
                            // todo: log it somehow
                            let _ = stream.respond(crate::err_400());
                            return;
                        };

                        let stream = Stream::new(stream, false);

                        router.respond(req, stream, state);
                    },
                );
            }
        }
    }

    /// Apply the provided function to every request.
    ///
    /// This option gives you maximum flexibility.
    ///
    /// note: if you choose this option, then the router will not be automatically applied to each
    /// request.
    pub fn for_each(self, func: fn(Request, Stream, STATE) -> UsedStream) {
        loop {
            if let Ok((stream, _)) = self.listener.accept() {
                let pointer = func as *const () as usize;

                let _ = process::spawn_with(
                    (pointer, stream.clone(), self.state.clone()),
                    |(pointer, stream, state), _: Mailbox<()>| {
                        let reconstructed_func = pointer as *const ();
                        let reconstructed_func = unsafe {
                            mem::transmute::<*const (), fn(Request, Stream, STATE) -> UsedStream>(
                                reconstructed_func,
                            )
                        };

                        let req = Request::parse(stream.clone());

                        match req {
                            Ok(Some(req)) => {
                                let stream = Stream::new(stream, false);

                                // todo: keep-alive
                                let _recovered_stream = (reconstructed_func)(req, stream, state);
                            }
                            _ => {
                                todo!()
                            }
                        }
                    },
                );
            }
        }
    }
}
///
#[derive(Debug)]
pub struct Stream {
    stream: TcpStream,
    /// Can this stream be kept alive once it is returned to the web server?
    ///
    /// If upgraded to a WebSocket connection, or the Content-Length is not
    /// specified by the client, then this is not possible.
    keep_alive: bool,
}

/// An error encountered when trying to upgrade a WebSocket connection.
#[derive(Debug)]
pub enum UpgradeError {
    /// Some other error not represented by another variant of this enum
    /// occured.
    __NonExhaustive,
}

impl Stream {
    // note: no keep_alive support for now!
    fn new(stream: TcpStream, keep_alive: bool) -> Stream {
        Self { stream, keep_alive }
    }

    /// Upgrade
    pub fn upgrade(mut self, req: &Request) -> Result<WebSocket, UpgradeError> {
        self.keep_alive = false;

        if !ws::should_upgrade(req) {
            return Err(UpgradeError::__NonExhaustive);
        }

        if !ws::perform_upgrade(req, self.stream.clone()) {
            return Err(UpgradeError::__NonExhaustive);
        }

        Ok(WebSocket::new(self.stream))
    }

    /// Send a response
    pub fn respond(self, response: Response) -> Result<UsedStream, io::Error> {
        let mut enc = Encoder::new(response);

        enc.write_tcp_stream(self.stream.clone())?;

        Ok(UsedStream {
            stream: self.stream,
            keep_alive: self.keep_alive,
        })
    }
}

#[derive(Debug)]
#[allow(unused)]
///
pub struct UsedStream {
    pub(crate) stream: TcpStream,
    pub(crate) keep_alive: bool,
}
