use serde::de::{self, value, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct FlakeLock {
    pub nodes: HashMap<String, Node>,
    pub root: String,
    pub version: u8,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Node {
    pub locked: Option<NodeLock>,
    pub original: Option<NodeRef>,
    #[serde(default)]
    pub inputs: HashMap<String, NodeInput>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeLock {
    pub last_modified: u32,
    pub nar_hash: String,
    #[serde(flatten)]
    pub reference: NodeRef,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum NodeRef {
    GitHub(NodeRefGitHub),
    Indirect(NodeRefIndirect),
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefGitHub {
    pub owner: String,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    #[serde(rename = "rev")]
    pub revision: Option<String>,
    pub repo: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefIndirect {
    pub id: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeInput(#[serde(deserialize_with = "string_or_vec")] pub Vec<String>);

/// From https://github.com/serde-rs/serde/issues/889#issuecomment-295988865
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![s.to_owned()])
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
