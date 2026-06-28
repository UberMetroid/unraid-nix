use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Structured result representing a found Nix package.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchResult {
    pub package_name: String,
    pub version: String,
    pub description: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub position: Option<String>,
}

/// Represents the raw JSON structure returned by 'nix search --json'.
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct NixSearchRawItem {
    pub pname: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Helper function to parse license info safely from Nix metadata.
fn parse_license(val: &serde_json::Value) -> Option<String> {
    if let Some(lic) = val.get("license") {
        if let Some(s) = lic.as_str() {
            return Some(s.to_string());
        }
        if let Some(obj) = lic.as_object() {
            if let Some(spdx) = obj.get("spdxId").and_then(|v| v.as_str()) {
                return Some(spdx.to_string());
            }
            if let Some(name) = obj.get("fullName").and_then(|v| v.as_str()) {
                return Some(name.to_string());
            }
            if let Some(short) = obj.get("shortName").and_then(|v| v.as_str()) {
                return Some(short.to_string());
            }
        }
        if let Some(arr) = lic.as_array() {
            let names: Vec<String> = arr.iter().filter_map(parse_license).collect();
            if !names.is_empty() {
                return Some(names.join(", "));
            }
        }
    }
    None
}

/// Formats the local nixpkgs store file position to a public GitHub link.
pub fn get_github_source_link(pos: &str) -> Option<String> {
    if let Some(idx) = pos.find("/pkgs/") {
        let rel_path = &pos[idx + 1..];
        let cleaned = rel_path.replace(':', "#L");
        return Some(format!(
            "https://github.com/NixOS/nixpkgs/blob/master/{cleaned}"
        ));
    }
    None
}

/// Performs a bulk 'nix eval' to fetch metadata for a list of raw attribute paths.
fn fetch_bulk_metadata(keys: &[String]) -> Result<HashMap<String, serde_json::Value>, String> {
    if keys.is_empty() {
        return Ok(HashMap::new());
    }

    let mut expr_parts = Vec::new();
    for key in keys {
        let rel_key = key.trim_start_matches("legacyPackages.x86_64-linux.");
        let path_segments: Vec<&str> = rel_key.split('.').collect();
        let path_expr = path_segments
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(" ");
        expr_parts.push(format!(
            "\"{}\" = (attrByPath [{}] {{}} lp).meta or {{}};",
            key, path_expr
        ));
    }

    let nix_expr = format!(
        "let pkgs = builtins.getFlake \"nixpkgs\"; lp = pkgs.legacyPackages.x86_64-linux; attrByPath = path: default: set: let len = builtins.length path; helper = idx: cur: if idx == len then cur else if builtins.isAttrs cur && builtins.hasAttr (builtins.elemAt path idx) cur then helper (idx + 1) (builtins.getAttr (builtins.elemAt path idx) cur) else default; in helper 0 set; in {{ {} }}",
        expr_parts.join(" ")
    );

    let output = Command::new("/nix/var/nix/profiles/default/bin/nix")
        .arg("eval")
        .arg("--impure")
        .arg("--json")
        .arg("--expr")
        .arg(&nix_expr)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run nix eval: {}", e))?;

    if !output.status.success() {
        return Ok(HashMap::new());
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&out_str)
        .map_err(|e| format!("Failed to parse metadata eval output: {}", e))?;

    Ok(parsed)
}

/// Parses the raw JSON map from 'nix search' and normalizes it.
pub fn parse_search_json(json_content: &str) -> Result<Vec<SearchResult>, String> {
    let raw_map: HashMap<String, NixSearchRawItem> = serde_json::from_str(json_content)
        .map_err(|e| format!("Failed to parse Nix search JSON: {}", e))?;

    let keys: Vec<String> = raw_map.keys().cloned().collect();
    let meta_map = fetch_bulk_metadata(&keys).unwrap_or_default();

    let mut results = Vec::new();
    for (key, item) in raw_map {
        let short_name = key.split('.').next_back().unwrap_or(&key);
        let normalized_name = format!("nixpkgs#{}", short_name);

        let mut homepage = None;
        let mut license = None;
        let mut position = None;

        if let Some(meta) = meta_map.get(&key) {
            homepage = meta
                .get("homepage")
                .and_then(|v| v.as_str())
                .map(String::from);
            license = parse_license(meta);
            position = meta
                .get("position")
                .and_then(|v| v.as_str())
                .and_then(get_github_source_link);
        }

        results.push(SearchResult {
            package_name: normalized_name,
            version: item.version.unwrap_or_default(),
            description: item.description.unwrap_or_default(),
            homepage,
            license,
            position,
        });
    }

    results.sort_by(|a, b| a.package_name.cmp(&b.package_name));
    Ok(results)
}
