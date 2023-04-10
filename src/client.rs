use std::{
    borrow::Borrow,
    io::{BufRead, BufReader, BufWriter, Stdin, Stdout, Write},
};

use serde::{Deserialize, Serialize};

use crate::{Error, Message, NodeId, Response};

#[derive()]
pub struct MaelstromClient {
    node_id: NodeId,
    node_ids: Vec<NodeId>,

    msg_id: Option<u32>,
    stdin: BufReader<Stdin>,
    stdout: BufWriter<Stdout>,
    buf: Vec<u8>,
}

impl MaelstromClient {
    pub fn new() -> Result<Self, Error> {
        let mut client = Self {
            node_id: NodeId::seq_kv(),
            node_ids: Vec::new(),
            msg_id: Some(0),
            stdin: BufReader::new(std::io::stdin()),
            stdout: BufWriter::new(std::io::stdout()),
            buf: Vec::new(),
        };

        client.handle_init()?;

        Ok(client)
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn node_ids(&self) -> &[NodeId] {
        &self.node_ids
    }

    pub fn message_id(&self) -> Option<u32> {
        self.msg_id
    }

    /// This client can only send messages without ids and cannot read
    pub fn detach(&self) -> MaelstromClient {
        MaelstromClient {
            node_id: self.node_id,
            node_ids: self.node_ids.clone(),
            msg_id: None,
            stdin: BufReader::new(std::io::stdin()),
            stdout: BufWriter::new(std::io::stdout()),
            buf: Vec::new(),
        }
    }

    pub fn read<'de, T: Deserialize<'de>>(&'de mut self) -> Result<Option<Message<T>>, Error> {
        if self.msg_id.is_none() {
            return Err(Error::DetachedClientCantRead);
        }

        self.buf.clear();

        let len = match self.stdin.read_until(b'\n', &mut self.buf) {
            Ok(len) => len,
            Err(err) => {
                dbg!(&err);
                return Err(err.into());
            }
        };
        if len == 0 {
            return Ok(None);
        }
        let buf = bstr::BStr::new(&self.buf[..len]);
        eprintln!("Read from stdin:");
        eprintln!("{buf}");
        let value = serde_json::from_slice(&self.buf[..len])?;
        Ok(Some(value))
    }

    fn write_<T: Serialize>(
        &mut self,
        resp: &Response<T>,
        needs_response: bool,
    ) -> Result<Option<u32>, Error> {
        let msg_id = if needs_response { self.msg_id } else { None };
        serde_json::to_writer(&mut self.stdout, &resp.raw(self.node_id, msg_id))?;
        self.stdout.write_all(b"\n")?;
        self.stdout.flush()?;

        if let Some(ref mut msg_id) = self.msg_id {
            *msg_id += 1;
        }

        Ok(msg_id)
    }

    pub fn write<T: Serialize>(&mut self, resp: impl Borrow<Response<T>>) -> Result<u32, Error> {
        self.write_(resp.borrow(), true).map(Option::unwrap)
    }

    pub fn write_no_response<T: Serialize>(
        &mut self,
        resp: impl Borrow<Response<T>>,
    ) -> Result<(), Error> {
        self.write_(resp.borrow(), false).map(drop)
    }

    fn handle_init(&mut self) -> Result<(), Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "lowercase")]
        pub enum InitPayload {
            Init {
                node_id: NodeId,
                node_ids: Vec<NodeId>,
            },
        }

        let init = self
            .read::<InitPayload>()?
            .ok_or(Error::MissingInitMessage)?;
        let resp = init.basic_response("init_ok");

        let InitPayload::Init { node_id, node_ids } = init.payload;
        self.node_id = node_id;
        self.node_ids = node_ids;

        self.write(resp)?;

        Ok(())
    }
}

#[allow(path_statements, clippy::no_effect)]
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {
        //
    }

    assert_send_sync::<MaelstromClient>;
    assert_send_sync::<Message<()>>;
    assert_send_sync::<Response<()>>;
};
