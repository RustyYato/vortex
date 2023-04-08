use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use vortex::MaelstromClient;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum EchoPayload<'a> {
    Echo { echo: Cow<'a, str> },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum EchoResponse<'a> {
    EchoOk { echo: &'a str },
}

pub fn main() -> anyhow::Result<()> {
    let mut client = MaelstromClient::new()?;

    while let Some(message) = client.read::<EchoPayload>()? {
        let EchoPayload::Echo { echo } = &message.payload;

        client.write(message.response(EchoResponse::EchoOk { echo }))?;
    }

    Ok(())
}
