#![feature(once_cell)]

use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, OnceLock},
};

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
    Gossip {
        gossip_id: u32,
        values: Vec<u32>,
    },
    GossipResponse {
        gossip_id: u32,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum BroadcastResponse<'a> {
    ReadOk { messages: &'a HashSet<u32> },
    Gossip { gossip_id: u32, values: Vec<u32> },
    GossipResponse { gossip_id: u32 },
}

struct State {
    values: HashSet<u32>,
    known: HashMap<NodeId, HashSet<u32>>,
    neighbors: Vec<NodeId>,
    gossip: HashMap<u32, Vec<u32>>,
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();

    STATE.get_or_init(|| {
        Mutex::new(State {
            values: HashSet::new(),
            known: HashMap::new(),
            gossip: HashMap::new(),
            neighbors: Vec::new(),
        })
    })
}

pub fn main() -> anyhow::Result<()> {
    let mut client = MaelstromClient::new()?;

    let state = state();

    let gossip_client = client.detach();

    std::thread::spawn(
        move || -> Result<core::convert::Infallible, anyhow::Error> {
            let mut client = gossip_client;
            let mut gossip_id = 0;

            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));

                let state = &mut *state.lock().unwrap();

                for &n in &state.neighbors {
                    let known = state.known.entry(n).or_default();

                    let values: Vec<u32> = state.values.difference(known).copied().collect();

                    if values.is_empty() {
                        continue;
                    }

                    gossip_id += 1;

                    state.gossip.insert(gossip_id, values.clone());

                    client.write(vortex::Response {
                        dest: n,
                        in_reply_to: None,
                        payload: BroadcastResponse::Gossip { gossip_id, values },
                    })?;
                }
            }
        },
    );

    while let Some(mut message) = client.read::<BroadcastPayload>()? {
        match message.payload {
            BroadcastPayload::Broadcast { message: value } => {
                state.lock().unwrap().values.insert(value);
                client.write(message.basic_response("broadcast_ok"))?;
            }
            BroadcastPayload::Read => {
                client.write(message.response(BroadcastResponse::ReadOk {
                    messages: &state.lock().unwrap().values,
                }))?;
            }
            BroadcastPayload::Topology { ref mut topology } => {
                state.lock().unwrap().neighbors = topology.remove(&client.node_id()).unwrap();
                client.write(message.basic_response("topology_ok"))?;
            }
            BroadcastPayload::Gossip {
                gossip_id,
                ref values,
            } => {
                let state = &mut *state.lock().unwrap();

                let known = state.known.entry(message.src).or_default();
                known.extend(values);
                state.values.extend(values);

                client.write(message.response(BroadcastResponse::GossipResponse { gossip_id }))?;
            }
            BroadcastPayload::GossipResponse { gossip_id } => {
                let state = &mut *state.lock().unwrap();

                let ref values = state.gossip.remove(&gossip_id).unwrap();

                let known = state.known.entry(message.src).or_default();
                known.extend(values);
                state.values.extend(values);
            }
        }
    }

    Ok(())
}