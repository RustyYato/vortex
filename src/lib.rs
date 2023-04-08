use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod client;
mod node_id;

pub use client::MaelstromClient;
pub use node_id::NodeId;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Deserialize(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("There must be an initialization message")]
    MissingInitMessage,
}

pub struct Message<Payload> {
    pub src: NodeId,
    pub dest: NodeId,
    pub msg_id: Option<u32>,
    pub in_reply_to: Option<u32>,
    pub payload: Payload,
}

#[derive(Serialize)]
pub struct BasicResponsePayload<'a> {
    #[serde(rename = "type")]
    pub ty: &'a str,
}

impl<T> Message<T> {
    pub fn response<U>(&self, payload: U) -> Response<U> {
        Response {
            dest: self.src,
            in_reply_to: self.msg_id,
            payload,
        }
    }

    pub fn basic_response<'a>(&self, ty: &'a str) -> Response<BasicResponsePayload<'a>> {
        self.response(BasicResponsePayload { ty })
    }
}

pub struct Response<Payload> {
    pub dest: NodeId,
    pub in_reply_to: Option<u32>,
    pub payload: Payload,
}

impl<Payload> Response<Payload> {
    pub fn with_payload<U>(self, payload: U) -> Response<U> {
        Response {
            dest: self.dest,
            in_reply_to: self.in_reply_to,
            payload,
        }
    }

    fn raw(&self, src: NodeId, msg_id: Option<u32>) -> RawResponse<&Payload> {
        RawResponse {
            src,
            dest: self.dest,
            body: RawResponseBody {
                msg_id,
                in_reply_to: self.in_reply_to,
                payload: &self.payload,
            },
        }
    }
}

impl<'de, Payload: Deserialize<'de>> Deserialize<'de> for Message<Payload> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct RawMessage<Payload> {
            pub src: NodeId,
            pub dest: NodeId,
            pub body: RawMessageBody<Payload>,
        }

        #[derive(Deserialize)]
        pub struct RawMessageBody<Payload> {
            pub msg_id: Option<u32>,
            pub in_reply_to: Option<u32>,
            #[serde(flatten)]
            pub payload: Payload,
        }

        let message = RawMessage::<Payload>::deserialize(deserializer)?;

        Ok(Self {
            src: message.src,
            dest: message.dest,
            msg_id: message.body.msg_id,
            in_reply_to: message.body.in_reply_to,
            payload: message.body.payload,
        })
    }
}

#[derive(Serialize)]
struct RawResponse<Payload> {
    pub src: NodeId,
    pub dest: NodeId,
    pub body: RawResponseBody<Payload>,
}

#[derive(Serialize)]
struct RawResponseBody<Payload> {
    pub msg_id: Option<u32>,
    pub in_reply_to: Option<u32>,
    #[serde(flatten)]
    pub payload: Payload,
}
