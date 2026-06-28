use crate::store;

pub fn setup_store(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Missing persistent path.");
        std::process::exit(1);
    }
    let path = &args[2];
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
    
    let cp_status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("mkdir -p {}/presets {}/presets_composed {}/presets {}/presets_composed && cp -rf {}/presets/* {}/presets/ 2>/dev/null; cp -rf {}/presets/* {}/presets/ 2>/dev/null; cp -rf {}/presets_composed/* {}/presets_composed/ 2>/dev/null; cp -rf {}/presets_composed/* {}/presets_composed/ 2>/dev/null", dest_usr, dest_usr, dest_boot, dest_boot, extracted_dir, dest_usr, extracted_dir, dest_boot, extracted_dir, dest_usr, extracted_dir, dest_boot))
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
