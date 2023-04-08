use serde::{Deserialize, Serialize};
use vortex::MaelstromClient;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum GeneratePayload {
    Generate,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum GenerateResponse<T> {
    GenerateOk { id: T },
}

pub fn main() -> anyhow::Result<()> {
    let mut client = MaelstromClient::new()?;

    while let Some(message) = client.read::<GeneratePayload>()? {
        client.write(message.response(GenerateResponse::GenerateOk {
            id: [client.node_id().value(), client.message_id()],
        }))?;
    }

    Ok(())
}
