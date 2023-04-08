use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    id: u32,
}

impl NodeId {
    pub(super) fn invalid() -> Self {
        Self { id: u32::MAX }
    }

    pub fn is_maelstrom_client(self) -> bool {
        self.id & 1 == 1
    }

    pub fn value(self) -> u32 {
        self.id >> 1
    }
}

impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(self.value());

        let mut output = [0u8; 1 + core::mem::size_of::<itoa::Buffer>()];
        output[0] = if self.is_maelstrom_client() {
            b'c'
        } else {
            b'n'
        };
        output[1..][..s.len()].copy_from_slice(s.as_bytes());
        let value = unsafe { core::str::from_utf8_unchecked(&output[..1 + s.len()]) };

        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NodeIdVisitor;

        impl<'de> serde::de::Visitor<'de> for NodeIdVisitor {
            type Value = NodeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a maelstrom node id")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Some(rest) = v.strip_prefix(['c', 'n']) {
                    let is_maelstrom_client = v.as_bytes()[0] == b'c';
                    let id = rest.parse::<u32>().map_err(|_| {
                        serde::de::Error::invalid_type(
                            serde::de::Unexpected::Str(v),
                            &"a maelstrtom node id",
                        )
                    })?;
                    Ok(NodeId {
                        id: id << 1 | (is_maelstrom_client as u32),
                    })
                } else {
                    Err(serde::de::Error::invalid_type(
                        serde::de::Unexpected::Str(v),
                        &"a node type either `c` or `n`",
                    ))
                }
            }
        }

        deserializer.deserialize_str(NodeIdVisitor)
    }
}
