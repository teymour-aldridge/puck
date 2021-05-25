use base64::encode;
use log::trace;
use sha1::{Digest, Sha1};

use crate::{err_400, lunatic::net::TcpStream, write_response, Response};

pub fn should_upgrade(req: &crate::Request) -> bool {
    req.headers
        .get("Upgrade")
        .map(|val| val == "WebSocket")
        .unwrap_or_default()
        && req
            .headers
            .get("Connection")
            .map(|val| val == "Upgrade")
            .unwrap_or_default()
}

/// The 'magic string' used to upgrade WebSocket connections.
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Tries to upgrade the connection to a WebSocket connection.
///
/// Returns true if this is successful, and false if it is not. Automatically sends a 400 Bad
/// Request response if the request fails.
pub fn perform_upgrade(req: &crate::Request, stream: TcpStream) -> bool {
    let key = match req.headers.get("Sec-WebSocket-Key") {
        Some(t) => t,
        None => {
            trace!("Rejecting WebSocket upgrade because of missing `Sec-WebSocket-Key` header.");
            write_response(err_400(), stream);
            return false;
        }
    };

    let result = compute_accept_header(key.clone());

    write_response(
        Response::build()
            .header("Sec-WebSocket-Accept", result)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .status(101, "Web Socket Protocol Handshake")
            .build(),
        stream,
    );

    true
}

fn compute_accept_header(key: String) -> String {
    let to_hash = key + GUID;

    let mut sha1 = Sha1::new();
    sha1.update(to_hash.as_bytes());
    let result = sha1.finalize();
    encode(result)
}

#[cfg(test)]
mod test {
    use lunatic::Process;

    use crate::ws::upgrade::compute_accept_header;

    fn test_compute_upgrade_header_inner(_: ()) {
        assert_eq!(
            compute_accept_header("dGhlIHNhbXBsZSBub25jZQ==".to_string()),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".to_string()
        );
    }

    #[test]
    fn test_compute_upgrade_header() {
        Process::spawn_with((), test_compute_upgrade_header_inner)
            .join()
            .unwrap();
    }
}
