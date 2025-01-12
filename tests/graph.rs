pub mod common;

use common::{bound_lock, simple_lock, LOOPED_LOCK_STR};
use flake_graph::{graph::NodeGraph, lock::FlakeLock};

#[test]
fn build_simple_lock_graph() {
    let lock = simple_lock();
    let graph = NodeGraph::from(lock);
    println!("{}", graph.to_dot());
}

#[test]
fn build_bound_lock_graph() {
    let lock = bound_lock();
    let graph = NodeGraph::from(lock);
    println!("{}", graph.to_dot());
}

#[test]
fn build_looped_lock_graph() {
    let parsed: FlakeLock = serde_json::from_str(LOOPED_LOCK_STR).unwrap();
    let graph = NodeGraph::from(parsed);
    println!("{}", graph.to_dot());
}
