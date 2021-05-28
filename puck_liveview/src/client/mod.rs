use serde::{Deserialize, Serialize};

mod send_changeset;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub(crate) listener: String,
    pub(crate) payload: Option<ClientMessagePayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessagePayload {
    pub(crate) value: String,
}
