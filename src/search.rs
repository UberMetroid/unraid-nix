/// Nix Package Search & Install Module
///
/// This module handles searching the nixpkgs registry using 'nix search'
/// and managing CLI packages in the user profile via 'nix profile'.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

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
            let names: Vec<String> = arr.iter()
                .filter_map(|item| parse_license(item))
                .collect();
            if !names.is_empty() {
                return Some(names.join(", "));
            }
        }
    }
    None
}

/// Formats the local nixpkgs store file position to a public GitHub link.
fn get_github_source_link(pos: &str) -> Option<String> {
    if let Some(idx) = pos.find("/pkgs/") {
        let rel_path = &pos[idx + 1..];
        let cleaned = rel_path.replace(':', "#L");
        return Some(format!("https://github.com/NixOS/nixpkgs/blob/master/{}", cleaned));
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
        let path_expr = path_segments.iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(" ");
        expr_parts.push(format!("\"{}\" = (attrByPath [{}] {{}} lp).meta or {{}};", key, path_expr));
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
///
/// Transforms internal Nix paths like 'legacyPackages.x86_64-linux.ripgrep'
/// into user-friendly installable names like 'nixpkgs#ripgrep'.
pub fn parse_search_json(json_content: &str) -> Result<Vec<SearchResult>, String> {
    let raw_map: HashMap<String, NixSearchRawItem> = serde_json::from_str(json_content)
        .map_err(|e| format!("Failed to parse Nix search JSON: {}", e))?;

    let keys: Vec<String> = raw_map.keys().cloned().collect();
    let meta_map = fetch_bulk_metadata(&keys).unwrap_or_default();

    let mut results = Vec::new();
    for (key, item) in raw_map {
        let short_name = key.split('.').last().unwrap_or(&key);
        let normalized_name = format!("nixpkgs#{}", short_name);

        let mut homepage = None;
        let mut license = None;
        let mut position = None;

        if let Some(meta) = meta_map.get(&key) {
            homepage = meta.get("homepage").and_then(|v| v.as_str()).map(|s| s.to_string());
            license = parse_license(meta);
            position = meta.get("position").and_then(|v| v.as_str()).and_then(get_github_source_link);
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

/// Searches the nixpkgs database for a package name or description.
pub fn search_packages(query: &str) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let output = Command::new("/nix/var/nix/profiles/default/bin/nix")
        .arg("search")
        .arg("--json")
        .arg("nixpkgs")
        .arg(query)
        .output()
        .map_err(|e| format!("Failed to run nix search: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Nix search error: {}", err_msg));
    }

    let json_content = String::from_utf8_lossy(&output.stdout);
    parse_search_json(&json_content)
}

/// Installs a package (CLI tool) into the user profile permanently.
pub fn install_package(package_name: &str) -> Result<(), String> {
    let status = Command::new("/nix/var/nix/profiles/default/bin/nix")
        .arg("profile")
        .arg("install")
        .arg(package_name)
        .status()
        .map_err(|e| format!("Failed to run nix profile install: {}", e))?;

    if !status.success() {
        return Err(format!("Failed to install package {}", package_name));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_json() {
        let mock_json = r#"{
            "legacyPackages.x86_64-linux.ripgrep": {
                "pname": "ripgrep",
                "version": "14.1.0",
                "description": "Fast grep replacement"
            },
            "legacyPackages.x86_64-linux.fd": {
                "pname": "fd",
                "version": "8.7.0",
                "description": "Simple alternative to find"
            }
        }"#;

        let results = parse_search_json(mock_json).unwrap();
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].package_name, "nixpkgs#fd");
        assert_eq!(results[0].version, "8.7.0");
        assert_eq!(results[0].description, "Simple alternative to find");

        assert_eq!(results[1].package_name, "nixpkgs#ripgrep");
        assert_eq!(results[1].version, "14.1.0");
        assert_eq!(results[1].description, "Fast grep replacement");
    }

    #[test]
    fn test_get_github_source_link() {
        let pos = "/nix/store/fj80w9qkzrk70bbs29l2b0107hlanj8p-source/pkgs/by-name/je/jellyfin/package.nix:59";
        let link = get_github_source_link(pos).unwrap();
        assert_eq!(link, "https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/je/jellyfin/package.nix#L59");
    }
}
