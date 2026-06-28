/// Helper to check if a directory contains any files.
pub fn has_files(dir: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        entries.count() > 0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_subdir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "nix-helper-has-files-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_has_files_empty_dir_returns_false() {
        let dir = temp_subdir("empty");
        assert!(!has_files(dir.to_str().unwrap()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_has_files_with_regular_file_returns_true() {
        let dir = temp_subdir("with-file");
        fs::write(dir.join("foo.txt"), "hello").unwrap();
        assert!(has_files(dir.to_str().unwrap()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_has_files_with_subdirectory_returns_true() {
        // Subdirectories still count as entries; the helper is named
        // `has_files` but really means "non-empty directory".
        let dir = temp_subdir("with-subdir");
        fs::create_dir(dir.join("nested")).unwrap();
        assert!(has_files(dir.to_str().unwrap()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_has_files_nonexistent_dir_returns_false() {
        // Missing directory must return false (not panic) so the caller can
        // treat "missing" the same as "empty".
        assert!(!has_files(
            "/this/path/definitely/does/not/exist/nix-helper"
        ));
    }

    #[test]
    fn test_has_files_symlink_loop_does_not_panic() {
        // Symlinks are read with `symlink_metadata` semantics by `read_dir`,
        // so the loop body doesn't follow them. We only need to confirm
        // the helper returns true (the entry exists) without infinite
        // recursion or panic.
        let dir = temp_subdir("symlink-loop");
        std::os::unix::fs::symlink(dir.join("loop"), dir.join("loop")).unwrap();
        assert!(has_files(dir.to_str().unwrap()));
        let _ = fs::remove_dir_all(&dir);
    }
}
