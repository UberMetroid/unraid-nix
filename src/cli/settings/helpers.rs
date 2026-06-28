/// Helper to check if a directory contains any files.
pub fn has_files(dir: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        entries.count() > 0
    } else {
        false
    }
}
