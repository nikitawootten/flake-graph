use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[serde(untagged)]
pub enum NodeInput {
    Direct(String),
    /// The path of inputs to follow from the `root` to the target
    Path(Vec<String>),
}
