use serde::{Deserialize, Serialize};
use vortex::{MaelstromClient, Response};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum GrowPayload {
    Add {
        delta: u32,
    },
    Read,
    #[serde(rename = "write_ok")]
    KvWriteOk,
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
    #[serde(rename = "write")]
    KvWriteInt {
        key: &'a str,
        value: u32,
    },
    #[serde(rename = "read")]
    KvReadInt {
        key: &'a str,
    },
}

pub fn main() -> anyhow::Result<()> {
    let mut client = MaelstromClient::new()?;

    let mut value = 0;

    client.write(Response {
        dest: vortex::NodeId::seq_kv(),
        in_reply_to: None,
        payload: GrowResponse::KvWriteInt {
            key: "hi",
            value: 0,
        },
    })?;

    client.write(Response {
        dest: vortex::NodeId::seq_kv(),
        in_reply_to: None,
        payload: GrowResponse::KvReadInt { key: "hi" },
    })?;

    while let Some(message) = client.read::<GrowPayload>()? {
        match message.payload {
            GrowPayload::Add { delta } => {
                value += delta;
                client.write(message.basic_response("add_ok"))?;
            }
            GrowPayload::Read => client.write(message.response(GrowResponse::ReadOk { value }))?,
            GrowPayload::KvWriteOk => (),
            GrowPayload::KvReadOk { value } => {
                eprintln!("READ {value}");
            }
        }
    }

    Ok(())
}
