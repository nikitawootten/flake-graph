use petgraph::{dot, prelude::DiGraph, stable_graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use petgraph::visit::EdgeRef;

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
            }),
            None => None,
        }
    }
}

type GraphT = DiGraph<Node, String>;

pub struct NodeGraph {
    pub graph: GraphT,
    pub root: NodeIndex,
    pub version: u8,
}

/// Process inputs for a node, creating graph edges
fn process_node_inputs<'a>(
    node_name: &String,
    flake_lock: &lock::FlakeLock,
    indices: &'a HashMap<String, NodeIndex>,
    mut graph: &'a mut GraphT,
    mut visited_nodes: &'a mut HashSet<NodeIndex>,
) -> (&'a mut GraphT, &'a mut HashSet<NodeIndex>) {
    let node_index = indices[node_name];
    if visited_nodes.contains(&node_index) {
        // Prevents duplicate edges
        return (graph, visited_nodes);
    }

    let raw_node = match flake_lock.nodes.get(node_name) {
        Some(node) => node,
        None => panic!("Node name does not exist in flake lock"),
    };
    // Create an edge for each node input
    for (input_edge_name, input) in &raw_node.inputs {
        // Inputs can point directly to a node by name or by a path of inputs
        match input {
            // Simple case, input is linked by name
            lock::NodeInput::Direct(input_node_name) => {
                graph.add_edge(
                    node_index,
                    indices[input_node_name],
                    input_edge_name.clone(),
                );
            }
            // Flake uses a "follows" directive, must traverse inputs down from root node
            lock::NodeInput::Path(input_path) => {
                if input_path.len() < 1 {
                    panic!("Input paths must have a length > 0");
                }
                let mut cursor = indices[&flake_lock.root];
                // Follow the path of inputs starting at the root
                for step_name in input_path {
                    // Check to see if the given node has been processed yet
                    if !visited_nodes.contains(&cursor) {
                        let cursor_node = match graph.node_weight(cursor) {
                            Some(node) => node,
                            // This should be unreachable™
                            None => panic!("Node could not be found"),
                        };
                        // If not, process it so we can follow its trail of inputs
                        (graph, visited_nodes) = process_node_inputs(
                            &cursor_node.name.clone(),
                            flake_lock,
                            indices,
                            graph,
                            visited_nodes,
                        );
                    }

                    let mut found_edge = false;
                    // Look for an edge with the correct input name
                    for edge in graph.edges(cursor) {
                        if *edge.weight() == *step_name {
                            // Advance the cursor down the path
                            cursor = edge.target();
                            found_edge = true;
                            break;
                        }
                    }

                    if !found_edge {
                        panic!("Edge path could not be followed")
                    }
                }
                graph.add_edge(node_index, cursor, input_edge_name.clone());
            }
        }
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
        // For each node in the flake lock:
        // 1. Create the graph node
        // 2. Insert the node into the graph
        for (key, raw_node) in &flake_lock.nodes {
            let node = graph.add_node(Node {
                name: key.clone(),
                original: raw_node.original.clone(),
                locked: raw_node.locked.clone(),
            });
            indices.insert(key.clone(), node);
        }

        let mut visited_nodes = &mut HashSet::<NodeIndex>::default();
        for node_name in flake_lock.nodes.keys() {
            (graph, visited_nodes) =
                process_node_inputs(node_name, &flake_lock, &indices, graph, visited_nodes);
        }

        Self {
            graph: std::mem::take(graph),
            root: indices[&flake_lock.root],
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
                            None => {
                                duplicates.insert(digest, vec![index]);
                            }
                        },
                        None => {}
                    };
                }
                None => {}
            });

        duplicates
    }

    pub fn to_dot<'a>(&self) -> String {
        let similarity_map = self.similarity_map();

        let node_labeller: &dyn Fn(_, (_, &Node)) -> String = &|_, (_, n)| {
            format!(
                "label = \"{}\"{}",
                n.name.clone(),
                match n.digest() {
                    Some(digest) => match similarity_map.get(&digest) {
                        Some(similarity) => match similarity.len() {
                            s if s > 1 => format!(" color={}", s),
                            _ => "".to_string(),
                        },
                        None => "".to_string(),
                    },
                    None => "".to_string(),
                }
            )
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
    node [colorscheme=oranges9]
{:?}
}}"#,
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
