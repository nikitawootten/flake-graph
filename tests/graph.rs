pub mod common;

use common::{bound_lock, simple_lock, LOOPED_LOCK_STR};
use flake_graph::{
    graph::{NodeGraph, SizeSummary},
    lock::FlakeLock,
    size,
};
use std::collections::HashMap;

#[test]
fn build_simple_lock_graph() {
    let lock = simple_lock();
    let graph = NodeGraph::from(lock);
    println!("{}", graph.to_dot(None));
}

#[test]
fn build_bound_lock_graph() {
    let lock = bound_lock();
    let graph = NodeGraph::from(lock);
    println!("{}", graph.to_dot(None));
}

#[test]
fn build_looped_lock_graph() {
    let parsed: FlakeLock = serde_json::from_str(LOOPED_LOCK_STR).unwrap();
    let graph = NodeGraph::from(parsed);
    println!("{}", graph.to_dot(None));
}

/// A locked node re-serializes into a clean `builtins.fetchTree` argument.
#[test]
fn locked_node_serializes_for_fetch_tree() {
    let nixpkgs = simple_lock().nodes.remove("nixpkgs").unwrap();
    let json = serde_json::to_string(&nixpkgs.locked.unwrap()).unwrap();
    assert!(json.contains("\"type\":\"github\""), "got: {}", json);
    assert!(json.contains("\"narHash\""), "got: {}", json);
    assert!(json.contains("\"rev\""), "got: {}", json);
}

/// When a size summary is supplied, node labels gain a size line and the graph gains a total.
#[test]
fn to_dot_renders_sizes() {
    let graph = NodeGraph::from(simple_lock());
    let nixpkgs = graph
        .graph
        .node_indices()
        .find(|i| graph.graph[*i].name == "nixpkgs")
        .unwrap();

    let summary = SizeSummary {
        per_node: HashMap::from([(nixpkgs, 1536)]),
        total: 1536,
        deduped_total: 1536,
        wasted: 0,
    };

    let dot = graph.to_dot(Some(&summary));
    assert!(dot.contains("1.5 KiB"), "node size missing in:\n{}", dot);
    assert!(
        dot.contains("total: 1.5 KiB"),
        "graph total missing in:\n{}",
        dot
    );
}

/// Ignored due to dependency on nix command and network.
#[test]
#[ignore]
fn source_size_resolves_real_input() {
    let nixpkgs = simple_lock().nodes.remove("nixpkgs").unwrap();
    let bytes = size::source_size(&nixpkgs.locked.unwrap()).expect("should resolve size");
    assert!(bytes > 0, "expected a non-zero source size");
}
