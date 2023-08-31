use flake_graph::lock::{FlakeLock, Node, NodeInput, NodeLock, NodeRef, NodeRefGitHub};
use std::collections::HashMap;

fn nixpkgs_node() -> Node {
    Node {
        locked: Some(NodeLock {
            last_modified: 1692742407,
            nar_hash: "sha256-faLzZ2u3Wki8h9ykEfzQr19B464eyADP3Ux7A/vjKIY=".to_string(),
            reference: NodeRef::GitHub(NodeRefGitHub {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
                revision: Some("a2eca347ae1e542af3f818274c38305c1e00604c".to_string()),
                reference: None,
            }),
        }),
        original: Some(NodeRef::GitHub(NodeRefGitHub {
            owner: "NixOS".to_string(),
            repo: "nixpkgs".to_string(),
            revision: None,
            reference: Some("nixpkgs-unstable".to_string()),
        })),
        inputs: HashMap::default(),
    }
}

pub const SIMPLE_LOCK_STR: &str = include_str!("./simple_flake.lock");

pub fn simple_lock() -> FlakeLock {
    return FlakeLock {
        root: "root".to_string(),
        version: 7,
        nodes: HashMap::from([
            ("nixpkgs".to_string(), nixpkgs_node()),
            (
                "root".to_string(),
                Node {
                    locked: None,
                    original: None,
                    inputs: HashMap::from([(
                        "nixpkgs".to_string(),
                        NodeInput(vec!["nixpkgs".to_string()]),
                    )]),
                },
            ),
        ]),
    };
}

pub const BOUND_LOCK_STR: &str = include_str!("./bound_flake.lock");

pub fn bound_lock() -> FlakeLock {
    return FlakeLock {
        root: "root".to_string(),
        version: 7,
        nodes: HashMap::from([
            (
                "home-manager".to_string(),
                Node {
                    locked: Some(NodeLock {
                        last_modified: 1693187908,
                        nar_hash: "sha256-cTcNpsqi1llmUFl9bmCdD0mTyfjhBrNFPhu2W12WXzA=".to_string(),
                        reference: NodeRef::GitHub(NodeRefGitHub {
                            owner: "nix-community".to_string(),
                            repo: "home-manager".to_string(),
                            revision: Some("8bde7a651b94ba30bd0baaa9c4a08aae88cc2e92".to_string()),
                            reference: None,
                        }),
                    }),
                    original: Some(NodeRef::GitHub(NodeRefGitHub {
                        owner: "nix-community".to_string(),
                        repo: "home-manager".to_string(),
                        revision: None,
                        reference: None,
                    })),
                    inputs: HashMap::from([(
                        "nixpkgs".to_string(),
                        NodeInput(vec!["nixpkgs".to_string()]),
                    )]),
                },
            ),
            ("nixpkgs".to_string(), nixpkgs_node()),
            (
                "root".to_string(),
                Node {
                    locked: None,
                    original: None,
                    inputs: HashMap::from([
                        (
                            "nixpkgs".to_string(),
                            NodeInput(vec!["nixpkgs".to_string()]),
                        ),
                        (
                            "home-manager".to_string(),
                            NodeInput(vec!["home-manager".to_string()]),
                        ),
                    ]),
                },
            ),
        ]),
    };
}
