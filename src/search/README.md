# search

Wraps `nix search` and `nix profile` for the WebUI. `mod.rs` exposes
`search_packages` (runs `nix search` and pipes its JSON output) and
`install_package` (runs `nix profile install`); `parser.rs` defines the
`SearchResult` struct and `parse_search_json`, which tolerates the two
JSON shapes `nix search` emits depending on version. Output is the
nixpkgs registry format — not nixos-options or flake outputs.
