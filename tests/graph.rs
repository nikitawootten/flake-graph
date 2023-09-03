pub mod common;

use common::{bound_lock, simple_lock};
use flake_graph::graph::NodeGraph;

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
