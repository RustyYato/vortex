use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use vortex::{MaelstromClient, Response};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum GrowPayload<'a> {
    Add {
        delta: u32,
    },
    Read,
    #[serde(rename = "cas_ok")]
    KvCasOk,
    Error {
        text: Cow<'a, str>,
    },
    #[serde(rename = "read_ok")]
    KvReadOk {
        value: u32,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum GrowResponse<'a> {
    ReadOk {
        value: u32,
    },
    #[serde(rename = "cas")]
    KvCas {
        key: &'a str,
        from: u32,
        to: u32,
        create_if_not_exists: bool,
    },
    #[serde(rename = "read")]
    KvRead {
        key: &'a str,
    },
}

const COUNTER: &str = "counter";

pub fn main() -> anyhow::Result<()> {
    let mut client = MaelstromClient::new()?;

    let mut value = 0;

    let mut cas_messages = Vec::new();
    let mut read_messages = Vec::new();

    while let Some(message) = client.read::<GrowPayload>()? {
        match message.payload {
            GrowPayload::Add { delta } => {
                client.write(message.basic_response("add_ok"))?;

                let msg_id = client.write(Response {
                    dest: vortex::NodeId::seq_kv(),
                    in_reply_to: None,
                    payload: GrowResponse::KvCas {
                        key: COUNTER,
                        from: value,
                        to: value + delta,
                        create_if_not_exists: true,
                    },
                })?;

                cas_messages.push((msg_id, delta));
            }
            GrowPayload::Read => {
                let id = client.write(Response {
                    dest: vortex::NodeId::seq_kv(),
                    in_reply_to: None,
                    payload: GrowResponse::KvRead { key: COUNTER },
                })?;

                read_messages.push((id, message.src, message.msg_id));
            }
            GrowPayload::KvReadOk { value } => {
                let Ok(id) = read_messages.binary_search_by_key(&message.in_reply_to, |&(id, ..)| Some(id)) else {
                    continue;
                };

                let (_, dest, in_reply_to) = read_messages.remove(id);

                client.write(Response {
                    dest,
                    in_reply_to,
                    payload: GrowResponse::ReadOk { value },
                })?;
            }
            GrowPayload::KvCasOk => {
                let resp = message.in_reply_to.unwrap();

                let Ok(msg) = cas_messages.binary_search_by_key(&resp, |&(msg_id, ..)| msg_id) else {
                    continue;
                };

                let (_, commited_delta) = cas_messages.remove(msg);
                value += commited_delta;
            }
            GrowPayload::Error { text } => {
                let resp = message.in_reply_to.unwrap();

                let Ok(msg) = cas_messages.binary_search_by_key(&resp, |&(msg_id, ..)| msg_id) else {
                    continue;
                };

                if let Some(text) = text.strip_prefix("current value ") {
                    let (current, _) = text.split_once(" is not ").unwrap();
                    value = current.parse::<u32>().unwrap();

                    let (_, delta) = cas_messages.remove(msg);

                    let msg_id = client.write(Response {
                        dest: vortex::NodeId::seq_kv(),
                        in_reply_to: None,
                        payload: GrowResponse::KvCas {
                            key: COUNTER,
                            from: value,
                            to: value + delta,
                            create_if_not_exists: true,
                        },
                    })?;
                    cas_messages.clear();
                    cas_messages.push((msg_id, delta));
                }
            }
        }
    }

    Ok(())
}
