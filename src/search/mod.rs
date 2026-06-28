//! Nix Package Search & Install Module
//!
//! This module handles searching the nixpkgs registry using 'nix search'
//! and managing CLI packages in the user profile via 'nix profile'.

pub mod parser;

pub use parser::{parse_search_json, SearchResult};

use std::process::{Command, Stdio};

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
        .stdin(Stdio::null())
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
        .stdin(Stdio::null())
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
        let link = parser::get_github_source_link(pos).unwrap();
        assert_eq!(
            link,
            "https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/je/jellyfin/package.nix#L59"
        );
    }
}
