use std::process::Command;
use super::helpers::has_files;

pub fn migrate_nix_store(clean_old_store_path: &str, clean_store_path: &str) -> bool {
    if std::path::Path::new(clean_old_store_path).exists() && has_files(clean_old_store_path) {
        // Stop services and unmount store prior to migration
        let _ = Command::new("/usr/local/emhttp/plugins/nix/event/stopping_svcs").output();
        let _ = Command::new("umount").args(["-l", "/nix"]).output();
        let _ = std::fs::create_dir_all(clean_store_path);
        
        // Sync files recursively preserving all attributes, ACLs, and hard links
        let _ = Command::new("rsync")
            .args(["-aHAX", &format!("{}/", clean_old_store_path), &format!("{}/", clean_store_path)])
            .output();
        true
    } else {
        false
    }
}
