use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vortex::{MaelstromClient, NodeId};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum BroadcastPayload {
    Broadcast {
        message: u32,
    },
    Read,
    Topology {
        topology: HashMap<NodeId, Vec<NodeId>>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum BroadcastResponse<'a> {
    ReadOk { messages: &'a [u32] },
}

pub fn main() -> anyhow::Result<()> {
    let mut client = MaelstromClient::new()?;

    let mut values = Vec::new();

    while let Some(message) = client.read::<BroadcastPayload>()? {
        match message.payload {
            BroadcastPayload::Broadcast { message: value } => {
                values.push(value);
                client.write(message.basic_response("broadcast_ok"))?;
            }
            BroadcastPayload::Read => {
                client.write(message.response(BroadcastResponse::ReadOk { messages: &values }))?;
            }
            BroadcastPayload::Topology { topology: _ } => {
                client.write(message.basic_response("topology_ok"))?;
            }
        }
    }

    Ok(())
}
