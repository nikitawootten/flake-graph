use flake_graph::lock::{
    FlakeLock, Node, NodeInput, NodeLock, NodeRef, NodeRefFile, NodeRefGit, NodeRefGitHub,
    NodeRefGitLab,
};
use std::collections::HashMap;

fn nixpkgs_node() -> Node {
    Node {
        locked: Some(NodeLock {
            last_modified: Some(1692742407),
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

/// JSON string of a simple lock file consisting of one input
pub const SIMPLE_LOCK_STR: &str = include_str!("./simple_flake.lock");

/// Struct representation of a simple lock file consisting of one input
pub fn simple_lock() -> FlakeLock {
    FlakeLock {
        root: "root".to_string(),
        version: 7,
        nodes: HashMap::from([
            ("nixpkgs".to_string(), nixpkgs_node()),
            (
                "root".to_string(),
                Node {
                    locked: None,
                    original: None,
                    inputs: HashMap::from([
                        (
                            "nixpkgs".to_string(),
                            NodeInput::Direct("nixpkgs".to_string()),
                        ),
                        ("yants".to_string(), NodeInput::Direct("yants".to_string())),
                    ]),
                },
            ),
            (
                "yants".to_string(),
                Node {
                    locked: Some(NodeLock {
                        last_modified: Some(1645270620),
                        nar_hash: "sha256-wwkl3K200UbW9Z7BRlVH8HOEXCaVYP2MqZpsF9EhgZg=".to_string(),
                        reference: NodeRef::Git(NodeRefGit {
                            url: "https://code.tvl.fyi/depot.git:/nix/yants.git".to_string(),
                            revision: Some("efeb6dc11eb1a1e88d41dc2093fc5aa31f7abd35".to_string()),
                            reference: Some("refs/heads/canon".to_string()),
                        }),
                    }),
                    original: Some(NodeRef::Git(NodeRefGit {
                        url: "https://code.tvl.fyi/depot.git:/nix/yants.git".to_string(),
                        revision: None,
                        reference: None,
                    })),
                    inputs: HashMap::default(),
                },
            ),
        ]),
    }
}

/// JSON string of a lock file with an input bound through a `follows` directive
pub const BOUND_LOCK_STR: &str = include_str!("./bound_flake.lock");

// Struct representation of a lock file with an input bound through a `follows` directive
pub fn bound_lock() -> FlakeLock {
    FlakeLock {
        root: "root".to_string(),
        version: 7,
        nodes: HashMap::from([
            (
                "home-manager".to_string(),
                Node {
                    locked: Some(NodeLock {
                        last_modified: Some(1693187908),
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
                        NodeInput::Path(vec!["nixpkgs".to_string()]),
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
                            NodeInput::Direct("nixpkgs".to_string()),
                        ),
                        (
                            "home-manager".to_string(),
                            NodeInput::Direct("home-manager".to_string()),
                        ),
                    ]),
                },
            ),
        ]),
    }
}

/// JSON string of a valid lock file with some missing fields in its inputs.
pub const MISSING_FIELDS_LOCK_STR: &str = include_str!("./missing_fields_flake.lock");

/// Struct representation of a valid lock file with some missing fields in its inputs.
pub fn missing_fields_lock() -> FlakeLock {
    FlakeLock {
        root: "root".to_string(),
        version: 7,
        nodes: HashMap::from([
            (
                "root".to_string(),
                Node {
                    locked: None,
                    original: None,
                    inputs: HashMap::from([
                        ("nmd".to_string(), NodeInput::Direct("nmd".to_string())),
                        (
                            "determinate-nixd-aarch64-darwin".to_string(),
                            NodeInput::Direct("determinate-nixd-aarch64-darwin".to_string()),
                        ),
                    ]),
                },
            ),
            (
                "nmd".to_string(),
                Node {
                    locked: Some(NodeLock {
                        last_modified: Some(1666190571),
                        nar_hash: "sha256-Z1hc7M9X6L+H83o9vOprijpzhTfOBjd0KmUTnpHAVjA=".to_string(),
                        reference: NodeRef::GitLab(NodeRefGitLab {
                            owner: "rycee".to_string(),
                            reference: None,
                            revision: Some("b75d312b4f33bd3294cd8ae5c2ca8c6da2afc169".to_string()),
                            repo: "nmd".to_string(),
                            host: "gitlab.com".to_string(),
                        }),
                    }),
                    original: Some(NodeRef::GitLab(NodeRefGitLab {
                        owner: "rycee".to_string(),
                        reference: None,
                        revision: None,
                        repo: "nmd".to_string(),
                        host: "gitlab.com".to_string(),
                    })),
                    inputs: HashMap::default(),
                },
            ),
            (
                "determinate-nixd-aarch64-darwin".to_string(),
                Node {
                    locked: Some(NodeLock {
                        last_modified: None,
                        nar_hash: "sha256-LNvx0qZsH8tbdgNfaig/x5Cf4r4UrXfU1m+0bO3D0E4=".to_string(),
                        reference: NodeRef::File(NodeRefFile {
                            url: "https://install.determinate.systems/determinate-nixd/tag/v3.21.0/macOS".to_string(),
                        }),
                    }),
                    original: Some(NodeRef::File(NodeRefFile {
                            url: "https://install.determinate.systems/determinate-nixd/tag/v3.21.0/macOS".to_string(),
                    })),
                    inputs: HashMap::default(),
                },
            ),
        ]),
    }
}

/// JSON string of a lock file with an input loop
pub const LOOPED_LOCK_STR: &str = include_str!("./looped_flake.lock");
