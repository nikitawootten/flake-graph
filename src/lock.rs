use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
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
    pub last_modified: Option<u32>,
    pub nar_hash: String,
    #[serde(flatten)]
    pub reference: NodeRef,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum NodeRef {
    Git(NodeRefGit),
    GitHub(NodeRefGitHub),
    GitLab(NodeRefGitLab),
    Indirect(NodeRefIndirect),
    Tarball(NodeRefTarball),
    File(NodeRefFile),
    Path(NodeRefPath),
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefGit {
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "rev", skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub url: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefGitHub {
    pub owner: String,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "rev", skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub repo: String,
}

fn gitlab_default_host() -> String {
    "gitlab.com".to_string()
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, SmartDefault)]
pub struct NodeRefGitLab {
    pub owner: String,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "rev", skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub repo: String,
    #[default = "gitlab.com"]
    #[serde(default = "gitlab_default_host")]
    pub host: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefIndirect {
    pub id: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefTarball {
    pub url: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefFile {
    pub url: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NodeRefPath {
    pub path: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum NodeInput {
    Direct(String),
    /// The path of inputs to follow from the `root` to the target
    Path(Vec<String>),
}
