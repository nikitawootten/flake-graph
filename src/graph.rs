use petgraph::{dot, prelude::DiGraph, stable_graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::lock;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Node {
    pub name: String,
    pub original: Option<lock::NodeRef>,
    pub locked: Option<lock::NodeLock>,
}

impl Node {
    fn digest(&self) -> Option<String> {
        match &self.locked {
            Some(locked) => Some(match &locked.reference {
                lock::NodeRef::GitHub(github) => {
                    format!("github::{}/{}", github.owner, github.repo)
                }
                lock::NodeRef::Indirect(indirect) => format!("indirect::{}", indirect.id),
                lock::NodeRef::Tarball(tarball) => format!("tarball::{}", tarball.url),
                lock::NodeRef::Path(path) => format!("path::{}", path.path),
            }),
            _ => None,
        }
    }
}

type GraphT = DiGraph<Node, String>;

pub struct NodeGraph {
    pub graph: GraphT,
    pub root: NodeIndex,
    pub version: u8,
}

fn traverse_path(path: Vec<String>, flake_lock: &lock::FlakeLock) -> String {
    let mut next_node_ref = flake_lock.root.clone();

    for step_name in path {
        let cursor = match flake_lock.nodes.get(&next_node_ref) {
            Some(node) => node,
            _ => panic!("Node '{}' does not exist in flake lock", next_node_ref),
        };

        next_node_ref = match cursor.inputs.get(&step_name) {
            Some(lock::NodeInput::Direct(next_node_name)) => next_node_name.clone(),
            Some(lock::NodeInput::Path(path)) => traverse_path(path.clone(), flake_lock),
            _ => panic!(
                "Could not traverse path, step '{}' does not exist",
                step_name
            ),
        };
    }

    next_node_ref
}

/// Process inputs for a node, creating graph edges
fn process_node_inputs<'a>(
    node_name: &String,
    flake_lock: &lock::FlakeLock,
    indices: &'a HashMap<String, NodeIndex>,
    graph: &'a mut GraphT,
    visited_nodes: &'a mut HashSet<NodeIndex>,
) -> (&'a mut GraphT, &'a mut HashSet<NodeIndex>) {
    let node_index = indices[node_name];
    if visited_nodes.contains(&node_index) {
        // Prevents duplicate edges
        return (graph, visited_nodes);
    }

    let raw_node = match flake_lock.nodes.get(node_name) {
        Some(node) => node,
        _ => panic!("Node '{}' does not exist", node_name),
    };
    // Create an edge for each node input
    for (input_edge_name, input) in &raw_node.inputs {
        // Inputs can point directly to a node by name or by a path of inputs
        let node_ref = match input {
            // Simple case, input is linked by name
            lock::NodeInput::Direct(input_node_name) => input_node_name.clone(),
            // Flake uses a "follows" directive, must traverse inputs down from root node
            lock::NodeInput::Path(input_path) => traverse_path(input_path.clone(), flake_lock),
        };

        let node = match indices.get(&node_ref) {
            Some(node) => node,
            _ => panic!(
                "Node '{}' has a non-existent input '{}'",
                node_name, node_ref
            ),
        };

        graph.add_edge(node_index, *node, input_edge_name.clone());
    }

    // Mark the edge as visited so that it is not processed twice
    visited_nodes.insert(node_index);

    return (graph, visited_nodes);
}

impl From<lock::FlakeLock> for NodeGraph {
    fn from(flake_lock: lock::FlakeLock) -> Self {
        let mut graph = &mut DiGraph::<Node, String>::new();

        // Map of node name -> graph node index
        let mut indices = HashMap::<String, NodeIndex>::new();
        log::trace!("Adding nodes to graph");
        for (key, raw_node) in &flake_lock.nodes {
            let node = graph.add_node(Node {
                name: key.clone(),
                original: raw_node.original.clone(),
                locked: raw_node.locked.clone(),
            });
            indices.insert(key.clone(), node);
        }

        log::trace!("Processing node inputs");
        let mut visited_nodes = &mut HashSet::<NodeIndex>::default();
        for node_name in flake_lock.nodes.keys() {
            log::trace!("Processing inputs for node {}", node_name);
            (graph, visited_nodes) =
                process_node_inputs(node_name, &flake_lock, &indices, graph, visited_nodes);
        }

        let root_node = match indices.get(&flake_lock.root) {
            Some(node) => node,
            _ => panic!(
                "Root node '{}' does not exist in flake lock",
                flake_lock.root
            ),
        };

        Self {
            graph: std::mem::take(graph),
            root: *root_node,
            version: flake_lock.version,
        }
    }
}

impl NodeGraph {
    pub fn similarity_map(&self) -> HashMap<String, Vec<NodeIndex>> {
        let mut duplicates = HashMap::<String, Vec<NodeIndex>>::new();
        self.graph
            .node_indices()
            .for_each(|index| match self.graph.node_weight(index) {
                Some(weight) => {
                    match weight.digest() {
                        Some(digest) => match duplicates.get_mut(&digest) {
                            Some(indices) => indices.push(index),
                            _ => {
                                duplicates.insert(digest, vec![index]);
                            }
                        },
                        _ => {}
                    };
                }
                _ => {}
            });

        duplicates
    }

    pub fn to_dot<'a>(&self) -> String {
        let similarity_map = self.similarity_map();

        let node_labeller: &dyn Fn(_, (_, &Node)) -> String = &|_, (_, n)| {
            let mut label = n.name.clone();
            let mut url: Option<String> = None;

            match &n.locked {
                Some(locked) => match &locked.reference {
                    lock::NodeRef::GitHub(github) => {
                        label.push_str(&format!("\\ngithub:{}/{}", github.owner, github.repo));
                        url = match &github.revision {
                            Some(rev) => Some(format!(
                                "https://github.com/{}/{}/tree/{}",
                                github.owner, github.repo, rev
                            )),
                            _ => Some(format!(
                                "https://github.com/{}/{}",
                                github.owner, github.repo
                            )),
                        };
                    }
                    _ => {}
                },
                _ => {}
            }

            let mut node_label = format!("label = \"{}\"", label,);

            if let Some(url) = url {
                node_label.push_str(&format!(", URL = \"{}\"", url));
            }

            if let Some(digest) = n.digest() {
                if let Some(similarity) = similarity_map.get(&digest) {
                    if similarity.len() > 1 {
                        node_label.push_str(&format!(", color={}", similarity.len()));
                    }
                }
            }

            node_label
        };

        let dot = dot::Dot::with_attr_getters(
            &self.graph,
            &[
                dot::Config::EdgeNoLabel,
                dot::Config::NodeNoLabel,
                dot::Config::GraphContentOnly,
            ],
            &|_, e| format!("label = \"{}\"", e.weight().clone()),
            node_labeller,
        );

        format!(
            r#"digraph {{
    node [colorscheme=oranges9 shape=record]
    rankdir=LR
{:?}}}"#,
            dot
        )
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
