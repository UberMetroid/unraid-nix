use crate::store;

pub fn setup_store(path: &str) {
    if let Err(e) = store::mount_nix_store(path) {
        eprintln!("Error mounting nix store: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = store::create_builder_accounts() {
        eprintln!("Error creating build accounts: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = store::setup_nix_conf() {
        eprintln!("Error setting up config: {}", e);
        std::process::exit(1);
    }
    println!("Nix Store successfully configured and mounted.");
}

pub fn teardown_store() {
    if let Err(e) = store::unmount_nix_store() {
        eprintln!("Error unmounting store: {}", e);
        std::process::exit(1);
    }
    println!("Nix Store successfully unmounted.");
}

pub fn sync_templates() {
    crate::store::log_event("INFO", "Syncing preset templates from unraid-nix-templates repository...");
    
    let zip_url = "https://github.com/UberMetroid/unraid-nix-templates/archive/refs/heads/main.zip";
    let tmp_zip = "/tmp/templates.zip";
    
    let curl_status = std::process::Command::new("curl")
        .args(["-sSf", "-L", "-o", tmp_zip, zip_url])
        .status();
        
    match curl_status {
        Ok(status) if status.success() => {},
        _ => {
            let err_msg = "Failed to download templates ZIP from GitHub.";
            crate::store::log_event("ERROR", err_msg);
            eprintln!("Error: {}", err_msg);
            std::process::exit(1);
        }
    }
    
    let tmp_dir = "/tmp";
    let unzip_status = std::process::Command::new("unzip")
        .args(["-q", "-o", tmp_zip, "-d", tmp_dir])
        .status();
        
    match unzip_status {
        Ok(status) if status.success() => {},
        _ => {
            let err_msg = "Failed to extract templates ZIP.";
            crate::store::log_event("ERROR", err_msg);
            eprintln!("Error: {}", err_msg);
            let _ = std::fs::remove_file(tmp_zip);
            std::process::exit(1);
        }
    }
    
    let extracted_dir = "/tmp/unraid-nix-templates-main";
    let dest_usr = "/usr/local/emhttp/plugins/nix";
    let dest_boot = "/boot/config/plugins/nix";

    // Sync presets to both the runtime plugin dir and the persistent flash
    // mirror. Prior versions of this command duplicated each `cp -rf` and
    // never actually copied into `dest_boot` — fix is to do four real copies.
    let cp_status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "mkdir -p {usr}/presets {usr}/presets_composed {boot}/presets {boot}/presets_composed \
             && cp -rf {src}/presets/* {usr}/presets/ 2>/dev/null \
             && cp -rf {src}/presets_composed/* {usr}/presets_composed/ 2>/dev/null \
             && cp -rf {src}/presets/* {boot}/presets/ 2>/dev/null \
             && cp -rf {src}/presets_composed/* {boot}/presets_composed/ 2>/dev/null",
            usr = dest_usr,
            boot = dest_boot,
            src = extracted_dir,
        ))
        .status();
        
    let _ = std::fs::remove_file(tmp_zip);
    let _ = std::fs::remove_dir_all(extracted_dir);
    
    match cp_status {
        Ok(status) if status.success() => {
            crate::store::log_event("INFO", "Templates successfully synced and updated.");
            println!("Templates successfully synced and updated.");
        },
        _ => {
            let err_msg = "Failed to copy templates to destination plugin directories.";
            crate::store::log_event("ERROR", err_msg);
            eprintln!("Error: {}", err_msg);
            std::process::exit(1);
        }
    }
}
