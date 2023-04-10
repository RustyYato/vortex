use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    imp: NodeIdImp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NodeIdImp {
    Maelstrom(u32),
    Node(u32),
    SeqKv,
}

impl NodeId {
    pub fn seq_kv() -> Self {
        Self {
            imp: NodeIdImp::SeqKv,
        }
    }

    pub fn is_seq_kv(self) -> bool {
        matches!(self.imp, NodeIdImp::SeqKv)
    }

    pub fn value(self) -> u32 {
        match self.imp {
            NodeIdImp::Maelstrom(value) | NodeIdImp::Node(value) => value,
            NodeIdImp::SeqKv => u32::MAX,
        }
    }
}

impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut output = [0u8; 1 + core::mem::size_of::<itoa::Buffer>()];
        output[0] = match self.imp {
            NodeIdImp::Maelstrom(_) => b'c',
            NodeIdImp::Node(_) => b'n',
            NodeIdImp::SeqKv => return "seq-kv".serialize(serializer),
        };
        let mut buf = itoa::Buffer::new();
        let s = buf.format(self.value());

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
                        imp: if is_maelstrom_client {
                            NodeIdImp::Maelstrom(id)
                        } else {
                            NodeIdImp::Node(id)
                        },
                    })
                } else if v == "seq-kv" {
                    Ok(NodeId {
                        imp: NodeIdImp::SeqKv,
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
