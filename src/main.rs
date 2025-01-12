use clap::Parser;
use flake_graph::{graph::NodeGraph, lock::FlakeLock};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the given flake.lock
    input: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let raw = std::fs::read_to_string(args.input).expect("Should have been able to read the file");
    let parsed: FlakeLock =
        serde_json::from_str(&raw).expect("Should have been able to parse flake lock");
    let graph = NodeGraph::from(parsed);
    println!("{}", graph.to_dot());
}
