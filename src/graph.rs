use petgraph::{prelude::DiGraph, stable_graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::lock::{self, FlakeLock};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Node {
    pub name: String,
    pub original: Option<lock::NodeRef>,
    pub locked: Option<lock::NodeLock>,
}

pub struct NodeGraph {
    pub graph: DiGraph<Node, String>,
    pub root: NodeIndex,
    pub version: u8,
}

impl From<lock::FlakeLock> for NodeGraph {
    fn from(flake_lock: FlakeLock) -> Self {
        let mut graph: DiGraph<Node, String> = DiGraph::new();

        let mut indices = HashMap::<String, NodeIndex>::new();
        for (key, raw_node) in &flake_lock.nodes {
            let node = graph.add_node(Node {
                name: key.clone(),
                original: raw_node.original.clone(),
                locked: raw_node.locked.clone(),
            });
            indices.insert(key.clone(), node);
        }

        for (a_key, raw_node) in flake_lock.nodes {
            for (e, b_keys) in raw_node.inputs {
                for b_key in b_keys.0 {
                    graph.add_edge(indices[&a_key], indices[&b_key], e.clone());
                }
            }
        }

        Self {
            graph,
            root: indices[&flake_lock.root],
            version: flake_lock.version,
        }
    }
}

impl Into<lock::FlakeLock> for NodeGraph {
    // TODO
    fn into(self) -> lock::FlakeLock {
        lock::FlakeLock {
            nodes: HashMap::default(),
            root: "".to_string(),
            version: self.version,
        }
    }
}
