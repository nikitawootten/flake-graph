pub mod common;

use common::{bound_lock, simple_lock};
use flake_graph::graph::NodeGraph;
use petgraph::dot::{Config, Dot};

#[test]
fn build_simple_lock_graph() {
    let lock = simple_lock();
    let graph = NodeGraph::from(lock);

    let dot = Dot::with_attr_getters(
        &graph.graph,
        &[Config::EdgeNoLabel, Config::NodeNoLabel],
        &|_, e| format!("label = \"{}\"", e.weight().clone()),
        &|_, (_, e)| format!("label = \"{}\"", e.name.clone()),
    );
    println!("{:?}", dot)
}

#[test]
fn build_bound_lock_graph() {
    let lock = bound_lock();
    let graph = NodeGraph::from(lock);

    let dot = Dot::with_attr_getters(
        &graph.graph,
        &[Config::EdgeNoLabel, Config::NodeNoLabel],
        &|_, e| format!("label = \"{}\"", e.weight().clone()),
        &|_, (_, e)| format!("label = \"{}\"", e.name.clone()),
    );
    println!("{:?}", dot)
}
