use petgraph::{prelude::DiGraph, stable_graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::lockfile::{self, FlakeLock};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Node {
    pub name: String,
    pub original: Option<lockfile::NodeRef>,
    pub locked: Option<lockfile::NodeLock>,
}

pub struct NodeGraph {
    pub graph: DiGraph<Node, String>,
    pub root: NodeIndex,
    pub version: u8,
}

impl From<lockfile::FlakeLock> for NodeGraph {
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

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::dot::{Config, Dot};

    // TODO common testing crate
    #[test]
    fn test() {
        let data = r#"
        {
            "nodes": {
                "nixpkgs": {
                    "locked": {
                        "lastModified": 1692742407,
                        "narHash": "sha256-faLzZ2u3Wki8h9ykEfzQr19B464eyADP3Ux7A/vjKIY=",
                        "owner": "NixOS",
                        "repo": "nixpkgs",
                        "rev": "a2eca347ae1e542af3f818274c38305c1e00604c",
                        "type": "github"
                    },
                    "original": {
                        "owner": "NixOS",
                        "ref": "nixpkgs-unstable",
                        "repo": "nixpkgs",
                        "type": "github"
                    }
              },
              "root": {
                    "inputs": {
                        "nixpkgs": "nixpkgs"
                    }
              }
            },
            "root": "root",
            "version": 7
        }
        "#;

        let parsed: FlakeLock = serde_json::from_str(data).unwrap();
        let graph = NodeGraph::from(parsed);
        let dot = Dot::with_attr_getters(
            &graph.graph,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_, e| format!("label = \"{}\"", e.weight().clone()),
            &|_, (_, e)| format!("label = \"{}\"", e.name.clone()),
        );
        println!("{:?}", dot)
    }
}
