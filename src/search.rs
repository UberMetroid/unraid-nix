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
}

/// Represents the raw JSON structure returned by 'nix search --json'.
#[derive(Deserialize, Debug)]
struct NixSearchRawItem {
    pub pname: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Parses the raw JSON map from 'nix search' and normalizes it.
///
/// Transforms internal Nix paths like 'legacyPackages.x86_64-linux.ripgrep'
/// into user-friendly installable names like 'nixpkgs#ripgrep'.
pub fn parse_search_json(json_content: &str) -> Result<Vec<SearchResult>, String> {
    let raw_map: HashMap<String, NixSearchRawItem> = serde_json::from_str(json_content)
        .map_err(|e| format!("Failed to parse Nix search JSON: {}", e))?;

    let mut results = Vec::new();
    for (key, item) in raw_map {
        // Extract the short package name from the last segment of the attribute path
        let short_name = key.split('.').last().unwrap_or(&key);
        let normalized_name = format!("nixpkgs#{}", short_name);

        results.push(SearchResult {
            package_name: normalized_name,
            version: item.version.unwrap_or_default(),
            description: item.description.unwrap_or_default(),
        });
    }

    // Sort results alphabetically by name
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

        // Verify alphabetical sorting
        assert_eq!(results[0].package_name, "nixpkgs#fd");
        assert_eq!(results[0].version, "8.7.0");
        assert_eq!(results[0].description, "Simple alternative to find");

        assert_eq!(results[1].package_name, "nixpkgs#ripgrep");
        assert_eq!(results[1].version, "14.1.0");
        assert_eq!(results[1].description, "Fast grep replacement");
    }
}
