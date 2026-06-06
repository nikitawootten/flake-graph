use clap::Parser;
use flake_graph::{graph::NodeGraph, lock::FlakeLock};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the given flake.lock
    input: String,

    /// Display the Nix store size of each input's source.
    ///
    /// Requires the `nix` binary and may fetch inputs from the network.
    #[arg(long)]
    size: bool,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let raw = std::fs::read_to_string(&args.input).expect("Should have been able to read the file");
    let parsed: FlakeLock =
        serde_json::from_str(&raw).expect("Should have been able to parse flake lock");
    let graph = NodeGraph::from(parsed);

    let sizes = if args.size {
        // The root flake's source is sized from the directory containing the flake.lock.
        let flake_dir = std::path::Path::new(&args.input)
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let summary = graph
            .source_sizes(&flake_dir.to_string_lossy())
            .unwrap_or_else(|err| panic!("Failed to compute input sizes: {}", err));
        eprintln!(
            "total source size: {} (deduplicated: {}, duplicated: {})",
            flake_graph::size::human_bytes(summary.total),
            flake_graph::size::human_bytes(summary.deduped_total),
            flake_graph::size::human_bytes(summary.wasted),
        );
        Some(summary)
    } else {
        None
    };

    println!("{}", graph.to_dot(sizes.as_ref()));
}
