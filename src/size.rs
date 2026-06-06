use std::collections::HashMap;
use std::fmt;
use std::process::Command;

use serde::Deserialize;

use crate::lock;

/// Experimental features required to use `nix eval`/`nix path-info`.
const EXPERIMENTAL_FEATURES: &str = "nix-command flakes";

/// Failure to resolve the size of an input's source.
#[derive(Debug)]
pub enum SizeError {
    /// The `nix` binary could not be found on `PATH`.
    NixNotFound,
    /// `nix eval` (fetching the source via `fetchTree`) failed.
    Eval(String),
    /// `nix path-info` failed or returned no entries.
    PathInfo(String),
    /// The JSON emitted by `nix` could not be parsed.
    Json(serde_json::Error),
}

impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeError::NixNotFound => write!(
                f,
                "the `nix` binary could not be found on PATH (required for --size)"
            ),
            SizeError::Eval(msg) => write!(f, "`nix eval` failed: {}", msg),
            SizeError::PathInfo(msg) => write!(f, "`nix path-info` failed: {}", msg),
            SizeError::Json(err) => write!(f, "could not parse nix JSON output: {}", err),
        }
    }
}

impl std::error::Error for SizeError {}

impl From<serde_json::Error> for SizeError {
    fn from(err: serde_json::Error) -> Self {
        SizeError::Json(err)
    }
}

/// A single entry of `nix path-info --json` output.
#[derive(Deserialize)]
struct PathInfo {
    #[serde(rename = "narSize")]
    nar_size: u64,
}

/// `nix path-info --json` output
#[derive(Deserialize)]
#[serde(untagged)]
enum PathInfoResponse {
    Array(Vec<PathInfo>),
    Map(HashMap<String, PathInfo>),
}

impl PathInfoResponse {
    fn nar_size(&self) -> Option<u64> {
        match self {
            PathInfoResponse::Array(entries) => entries.first().map(|e| e.nar_size),
            PathInfoResponse::Map(entries) => entries.values().next().map(|e| e.nar_size),
        }
    }
}

fn escape_nix_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace("${", "\\${")
}

/// Run `nix` with the given arguments, returning stdout on success.
fn run_nix(args: &[&str]) -> Result<String, SizeError> {
    let output = Command::new("nix").args(args).output().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            SizeError::NixNotFound
        } else {
            SizeError::Eval(err.to_string())
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SizeError::Eval(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Relevant fields of `nix flake metadata --json` output.
#[derive(Deserialize)]
struct FlakeMetadata {
    /// Store path of the flake's source tree.
    path: String,
}

/// Read the source's own NAR size of a store path via `nix path-info --json`.
fn path_info_size(store_path: &str) -> Result<u64, SizeError> {
    let raw = run_nix(&[
        "path-info",
        "--json",
        "--extra-experimental-features",
        EXPERIMENTAL_FEATURES,
        store_path,
    ])
    .map_err(|err| match err {
        SizeError::Eval(msg) => SizeError::PathInfo(msg),
        other => other,
    })?;

    let response: PathInfoResponse = serde_json::from_str(&raw)?;
    response
        .nar_size()
        .ok_or_else(|| SizeError::PathInfo(format!("no entry for store path {}", store_path)))
}

/// Resolve the store path of a locked reference and return its source's own size in bytes.
///
/// The locked reference is fetched into the store if not already present.
pub fn source_size(locked: &lock::NodeLock) -> Result<u64, SizeError> {
    let json = serde_json::to_string(locked)?;
    let expr = format!(
        "(builtins.fetchTree (builtins.fromJSON \"{}\")).outPath",
        escape_nix_string(&json)
    );

    let store_path = run_nix(&[
        "eval",
        "--raw",
        "--extra-experimental-features",
        EXPERIMENTAL_FEATURES,
        "--expr",
        &expr,
    ])?;

    path_info_size(store_path.trim())
}

/// Resolve the source size of the root node (local flake) from its directory.
///
/// Unlike locked inputs, the root flake is not described by `flake.lock`, so its source is
/// resolved via `nix flake metadata`, which reports the store path of the flake source (the
/// git-tracked tree, copied into the store).
pub fn flake_source_size(flake_dir: &str) -> Result<u64, SizeError> {
    let raw = run_nix(&[
        "flake",
        "metadata",
        "--json",
        "--extra-experimental-features",
        EXPERIMENTAL_FEATURES,
        flake_dir,
    ])?;

    let metadata: FlakeMetadata = serde_json::from_str(&raw)?;
    path_info_size(&metadata.path)
}

/// Format a byte count as a short human-readable string using IEC (binary) units, matching
/// Nix's own conventions (e.g. `1536` -> `"1.5 KiB"`).
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_boundaries() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1023), "1023 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1536), "1.5 KiB");
        assert_eq!(human_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(human_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn escape_nix_string_specials() {
        assert_eq!(escape_nix_string(r#"a"b"#), r#"a\"b"#);
        assert_eq!(escape_nix_string(r"a\b"), r"a\\b");
        assert_eq!(escape_nix_string("a${b}"), "a\\${b}");
    }

    #[test]
    fn path_info_parses_array_form() {
        let raw = r#"[{"path":"/nix/store/x","narSize":4096,"other":1}]"#;
        let parsed: PathInfoResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.nar_size(), Some(4096));
    }

    #[test]
    fn path_info_parses_map_form() {
        let raw = r#"{"/nix/store/x":{"narSize":8192,"other":1}}"#;
        let parsed: PathInfoResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.nar_size(), Some(8192));
    }
}
