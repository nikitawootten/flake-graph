use petgraph::{dot, prelude::DiGraph, stable_graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::lock;
use crate::size::{self, SizeError};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Node {
    pub name: String,
    pub original: Option<lock::NodeRef>,
    pub locked: Option<lock::NodeLock>,
}

impl Node {
    fn digest(&self) -> Option<String> {
        self.locked.as_ref().map(|locked| match &locked.reference {
            lock::NodeRef::Git(git) => format!("git::{}", git.url),
            lock::NodeRef::GitHub(github) => {
                format!("github::{}/{}", github.owner, github.repo)
            }
            lock::NodeRef::GitLab(gitlab) => {
                format!("gitlab::{}/{}@{}", gitlab.owner, gitlab.repo, gitlab.host)
            }
            lock::NodeRef::Indirect(indirect) => format!("indirect::{}", indirect.id),
            lock::NodeRef::Tarball(tarball) => format!("tarball::{}", tarball.url),
            lock::NodeRef::Path(path) => format!("path::{}", path.path),
        })
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

    (graph, visited_nodes)
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

/// Source sizes for a [`NodeGraph`], keyed by graph node index.
#[derive(Debug, Default, PartialEq)]
pub struct SizeSummary {
    /// The source size in bytes for each node.
    pub per_node: HashMap<NodeIndex, u64>,
    /// Sum of every node's source size (duplicated sources counted once per node).
    pub total: u64,
    /// Sum of each distinct source's size (duplicated sources counted once).
    pub deduped_total: u64,
    /// Bytes attributable to duplicated sources (`total - deduped_total`).
    pub wasted: u64,
}

impl NodeGraph {
    /// Resolve the source size of every node.
    pub fn source_sizes(&self, flake_dir: &str) -> Result<SizeSummary, SizeError> {
        let mut summary = SizeSummary::default();

        for indices in self.similarity_map().values() {
            let locked = indices
                .iter()
                .find_map(|index| self.graph.node_weight(*index))
                .and_then(|node| node.locked.as_ref());
            let locked = match locked {
                Some(locked) => locked,
                None => continue,
            };

            log::trace!("Resolving source size for digest shared by {:?}", indices);
            let size = size::source_size(locked)?;

            summary.deduped_total += size;
            for index in indices {
                summary.per_node.insert(*index, size);
                summary.total += size;
            }
        }

        // The root node needs to be sized separately
        if let std::collections::hash_map::Entry::Vacant(entry) = summary.per_node.entry(self.root)
        {
            log::trace!("Resolving source size for root flake at {}", flake_dir);
            let size = size::flake_source_size(flake_dir)?;
            entry.insert(size);
            summary.total += size;
            summary.deduped_total += size;
        }

        summary.wasted = summary.total - summary.deduped_total;
        Ok(summary)
    }

    pub fn similarity_map(&self) -> HashMap<String, Vec<NodeIndex>> {
        let mut duplicates = HashMap::<String, Vec<NodeIndex>>::new();
        self.graph.node_indices().for_each(|index| {
            if let Some(weight) = self.graph.node_weight(index) {
                if let Some(digest) = weight.digest() {
                    match duplicates.get_mut(&digest) {
                        Some(indices) => indices.push(index),
                        _ => {
                            duplicates.insert(digest, vec![index]);
                        }
                    }
                };
            }
        });

        duplicates
    }

    pub fn to_dot(&self, sizes: Option<&SizeSummary>) -> String {
        let similarity_map = self.similarity_map();

        let node_labeller: &dyn Fn(_, (NodeIndex, &Node)) -> String = &|_, (idx, n)| {
            let mut label = n.name.clone();
            let mut url: Option<String> = None;

            if let Some(locked) = &n.locked {
                if let lock::NodeRef::GitHub(github) = &locked.reference {
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
                } else if let lock::NodeRef::GitLab(gitlab) = &locked.reference {
                    label.push_str(&format!(
                        "\\n{}:{}/{}",
                        gitlab.host, gitlab.owner, gitlab.repo
                    ));
                    url = match &gitlab.revision {
                        Some(rev) => Some(format!(
                            "https://{}/{}/{}/-/tree/{}",
                            gitlab.host, gitlab.owner, gitlab.repo, rev
                        )),
                        _ => Some(format!(
                            "https://{}/{}/{}",
                            gitlab.host, gitlab.owner, gitlab.repo
                        )),
                    };
                }
            }

            if let Some(sizes) = sizes {
                if let Some(bytes) = sizes.per_node.get(&idx) {
                    label.push_str(&format!("\\n{}", size::human_bytes(*bytes)));
                }
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

        let graph_label = match sizes {
            Some(sizes) => format!(
                "    label=\"total: {}  (duplicated: {})\"\n    labelloc=t\n",
                size::human_bytes(sizes.total),
                size::human_bytes(sizes.wasted)
            ),
            None => String::new(),
        };

        format!(
            r#"digraph {{
    node [colorscheme=oranges9 shape=record]
    rankdir=LR
{}{:?}}}"#,
            graph_label, dot
        )
    }
}

impl From<NodeGraph> for lock::FlakeLock {
    // TODO
    fn from(val: NodeGraph) -> Self {
        lock::FlakeLock {
            nodes: HashMap::default(),
            root: "".to_string(),
            version: val.version,
        }
    }
}
