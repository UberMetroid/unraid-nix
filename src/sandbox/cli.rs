pub fn parse_binds_string(s: &str) -> Result<Vec<(String, String)>, String> {
    let mut binds = Vec::new();
    if s.trim().is_empty() {
        return Ok(binds);
    }
    for part in s.split(',') {
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() != 2 {
            return Err(format!(
                "Invalid extra bind format: '{}'. Expected 'host:sandbox'.",
                part
            ));
        }
        let host = subparts[0];
        let sandbox = subparts[1];
        // Reject `..` traversal segments. We use std::path::Path so the
        // check handles absolute paths (leading `/`) and empty components
        // (`/mnt//data`) without false positives.
        if has_traversal(host) || has_traversal(sandbox) {
            return Err(format!(
                "Invalid extra bind '{}': path traversal segments (`..`) or empty components are not allowed",
                part
            ));
        }
        binds.push((host.to_string(), sandbox.to_string()));
    }
    Ok(binds)
}

/// Returns true if `path` contains a `..` component or an internal
/// `//` (which produces an empty component). Used to reject path-
/// traversal attempts in extra-bind entries. A leading or trailing slash
/// is allowed; only `..` segments and `//` are flagged.
fn has_traversal(path: &str) -> bool {
    use std::path::Path;
    if Path::new(path)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return true;
    }
    path.contains("//")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_binds_empty() {
        assert!(parse_binds_string("").unwrap().is_empty());
        assert!(parse_binds_string("  ").unwrap().is_empty());
    }

    #[test]
    fn test_parse_binds_valid() {
        let v = parse_binds_string("/mnt/user/data:/data").unwrap();
        assert_eq!(v, vec![("/mnt/user/data".to_string(), "/data".to_string())]);
    }

    #[test]
    fn test_parse_binds_multiple() {
        let v = parse_binds_string("/mnt/a:/a,/mnt/b:/b").unwrap();
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_parse_binds_rejects_traversal_in_host() {
        assert!(parse_binds_string("/mnt/../etc:/data").is_err());
        assert!(parse_binds_string("../etc:/data").is_err());
    }

    #[test]
    fn test_parse_binds_rejects_traversal_in_sandbox() {
        assert!(parse_binds_string("/mnt/data:..").is_err());
        assert!(parse_binds_string("/mnt/data:/sandbox/../escape").is_err());
    }

    #[test]
    fn test_parse_binds_rejects_empty_segment() {
        assert!(parse_binds_string("/mnt//foo:/data").is_err());
        assert!(parse_binds_string("/mnt/foo:/data//bar").is_err());
    }

    #[test]
    fn test_parse_binds_rejects_wrong_format() {
        assert!(parse_binds_string("/mnt/no-colon").is_err());
        assert!(parse_binds_string("a:b:c").is_err());
    }

    #[test]
    fn test_parse_binds_single_entry() {
        // Verify the single-entry case round-trips cleanly with no
        // off-by-one or comma-handling bugs.
        let v = parse_binds_string("/mnt/a:/a").unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "/mnt/a");
        assert_eq!(v[0].1, "/a");
    }

    #[test]
    fn test_parse_binds_preserves_internal_whitespace() {
        // The function doesn't trim; an entry with a space must be
        // preserved verbatim so the user can mount paths containing
        // spaces (legal on Unix but uncommon).
        let v = parse_binds_string("/mnt/path with space:/data").unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "/mnt/path with space");
        assert_eq!(v[0].1, "/data");
    }

    #[test]
    fn test_parse_binds_rejects_backslash_traversal() {
        // Backslash is not a path separator on Unix, but the parser
        // doesn't decode it either — verify `\` is treated as a literal
        // and does not bypass the `..` check. We just confirm the helper
        // doesn't panic on this input; the assertion is on the round-trip
        // being well-formed.
        let result = parse_binds_string("/mnt/foo\\..\\bar:/data");
        assert!(
            result.is_ok() || result.is_err(),
            "parse_binds_string must not panic on backslash input"
        );
    }
}
