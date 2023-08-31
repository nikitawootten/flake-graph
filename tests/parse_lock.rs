pub mod common;

use common::{bound_lock, simple_lock, BOUND_LOCK_STR, SIMPLE_LOCK_STR};
use flake_graph::lock::FlakeLock;

#[test]
fn parse_simple_lock() {
    let parsed: FlakeLock = serde_json::from_str(SIMPLE_LOCK_STR).unwrap();
    assert_eq!(parsed, simple_lock());
}

#[test]
fn parse_bound_lock() {
    let parsed: FlakeLock = serde_json::from_str(BOUND_LOCK_STR).unwrap();
    assert_eq!(parsed, bound_lock());
}
