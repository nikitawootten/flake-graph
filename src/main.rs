// TODO build out subcommands

use flake_graph::{graph::NodeGraph, lock::FlakeLock};
use petgraph::dot;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        2 => {
            let filename = &args[1];
            let raw =
                std::fs::read_to_string(filename).expect("Should have been able to read the file");
            let parsed: FlakeLock =
                serde_json::from_str(&raw).expect("Should have been able to parse flake lock");
            let graph = NodeGraph::from(parsed);
            let dot = dot::Dot::with_attr_getters(
                &graph.graph,
                &[dot::Config::EdgeNoLabel, dot::Config::NodeNoLabel],
                &|_, e| format!("label = \"{}\"", e.weight().clone()),
                &|_, (_, e)| format!("label = \"{}\"", e.name.clone()),
            );
            println!("{:?}", dot)
        }
        _ => println!("usage: <input flake.lock>"),
    };
}
