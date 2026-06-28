use crate::store;

pub fn setup_store(path: &str) {
    if let Err(e) = store::mount_nix_store(path) {
        crate::store::log_event("ERROR", &format!("Mounting nix store failed: {e}"));
        std::process::exit(1);
    }
    if let Err(e) = store::create_builder_accounts() {
        crate::store::log_event("ERROR", &format!("Creating build accounts failed: {e}"));
        std::process::exit(1);
    }
    if let Err(e) = store::setup_nix_conf() {
        crate::store::log_event("ERROR", &format!("Setting up nix.conf failed: {e}"));
        std::process::exit(1);
    }
    println!("Nix Store successfully configured and mounted.");
}

pub fn teardown_store() {
    if let Err(e) = store::unmount_nix_store() {
        crate::store::log_event("ERROR", &format!("Unmounting nix store failed: {e}"));
        std::process::exit(1);
    }
    println!("Nix Store successfully unmounted.");
}

pub fn sync_templates() {
    crate::store::log_event(
        "INFO",
        "Syncing preset templates from unraid-nix-templates repository...",
    );

    let cp_status =
        std::process::Command::new("/usr/local/emhttp/plugins/nix/scripts/sync-templates.sh")
            .status();

    match cp_status {
        Ok(status) if status.success() => {
            crate::store::log_event("INFO", "Templates successfully synced and updated.");
            println!("Templates successfully synced and updated.");
        }
        _ => {
            let err_msg = "Failed to sync templates via sync-templates.sh.";
            crate::store::log_event("ERROR", err_msg);
            std::process::exit(1);
        }
    }
}
