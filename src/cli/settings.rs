use std::collections::HashMap;
use std::process::exit;

/// Parses a simple ini-like configuration file into a key-value HashMap.
/// Ignores empty lines and comments starting with ';' or '#'.
pub fn parse_ini_file(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let val = line[pos + 1..].trim().trim_matches('"').to_string();
                map.insert(key, val);
            }
        }
    }
    map
}

/// Detects the default Nix store location on Unraid.
/// Checks the system share configurations to locate the preferred cache pool,
/// falling back to `/mnt/user/system/nix` if no cache pool is defined.
pub fn detect_default_store_path() -> String {
    let system_cfg = parse_ini_file("/boot/config/shares/system.cfg");
    let mut pool = system_cfg.get("shareCachePool").cloned().unwrap_or_default();
    if pool.is_empty() {
        if let Some(use_cache) = system_cfg.get("shareUseCache") {
            if use_cache == "yes" || use_cache == "prefer" || use_cache == "only" {
                pool = "cache".to_string();
            }
        }
    }
    if !pool.is_empty() {
        let path = format!("/mnt/{}/system/nix", pool);
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/system").is_dir() {
        return "/mnt/user/system/nix".to_string();
    }
    "".to_string()
}

/// Detects the Appdata directory root path on Unraid.
/// Resolves pool-specific appdata paths (e.g. `/mnt/cache/appdata`)
/// before falling back to the default `/mnt/user/appdata`.
pub fn detect_appdata_root() -> String {
    let appdata_cfg = parse_ini_file("/boot/config/shares/appdata.cfg");
    let mut pool = appdata_cfg.get("shareCachePool").cloned().unwrap_or_default();
    if pool.is_empty() {
        if let Some(use_cache) = appdata_cfg.get("shareUseCache") {
            if use_cache == "yes" || use_cache == "prefer" || use_cache == "only" {
                pool = "cache".to_string();
            }
        }
    }
    if !pool.is_empty() {
        let path = format!("/mnt/{}/appdata", pool);
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/appdata").is_dir() {
        return "/mnt/user/appdata".to_string();
    }
    "".to_string()
}

/// Checks if a directory exists and contains any files.
fn has_files(dir: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        entries.count() > 0
    } else {
        false
    }
}

/// Saves the Nix plugin settings, handles the migration of /nix store paths
/// using rsync, and updates the Unraid web interface navigation registry.
pub fn save_settings(args: &[String]) {
    let mut store_path = String::new();
    let mut autostart = "yes".to_string();
    let mut enable_sandbox = "no".to_string();
    let mut enable_cli = "no".to_string();
    let mut show_in_nav = "yes".to_string();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--store-path" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing store path"); exit(1); }
                store_path = args[i+1].clone();
                i += 2;
            }
            "--autostart" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing autostart"); exit(1); }
                autostart = args[i+1].clone();
                i += 2;
            }
            "--enable-sandbox" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing enable-sandbox"); exit(1); }
                enable_sandbox = args[i+1].clone();
                i += 2;
            }
            "--enable-cli" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing enable-cli"); exit(1); }
                enable_cli = args[i+1].clone();
                i += 2;
            }
            "--show-in-nav" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing show-in-nav"); exit(1); }
                show_in_nav = args[i+1].clone();
                i += 2;
            }
            _ => { eprintln!("Unknown save-settings flag: {}", args[i]); exit(1); }
        }
    }

    let cfg_file = "/boot/config/plugins/nix/nix.cfg";
    let old_cfg = parse_ini_file(cfg_file);
    let mut old_store_path = old_cfg.get("NIX_STORE_PATH").cloned().unwrap_or_default();
    if old_store_path.is_empty() {
        old_store_path = detect_default_store_path();
    }

    let clean_store_path = store_path.trim_end_matches('/').to_string();
    let clean_old_store_path = old_store_path.trim_end_matches('/').to_string();

    let mut migration_performed = false;
    // Perform /nix store data migration if the user configured a new location
    if !clean_store_path.is_empty() && clean_store_path != clean_old_store_path {
        if std::path::Path::new(&clean_old_store_path).exists() && has_files(&clean_old_store_path) {
            // Stop services and unmount store prior to migration
            let _ = std::process::Command::new("/usr/local/emhttp/plugins/nix/event/stopping_svcs").output();
            let _ = std::process::Command::new("umount").args(&["-l", "/nix"]).output();
            let _ = std::fs::create_dir_all(&clean_store_path);
            
            // Sync files recursively preserving all attributes, ACLs, and hard links
            let _ = std::process::Command::new("rsync")
                .args(&["-aHAX", &format!("{}/", clean_old_store_path), &format!("{}/", clean_store_path)])
                .output();
            migration_performed = true;
        }
    }

    // Write settings to ini config
    let _ = std::fs::create_dir_all("/boot/config/plugins/nix");
    let cfg_content = format!(
        "NIX_STORE_PATH=\"{}\"\nAUTOSTART_FLAKES=\"{}\"\nENABLE_STORAGE_SANDBOX=\"{}\"\nENABLE_CLI_INSTALL=\"{}\"\nSHOW_IN_NAVIGATION=\"{}\"\n",
        clean_store_path, autostart, enable_sandbox, enable_cli, show_in_nav
    );
    if std::fs::write(cfg_file, cfg_content).is_err() {
        eprintln!("Failed to write nix.cfg to flash drive.");
        exit(1);
    }

    // Update Unraid side navigation config registry
    let nix_page_file = "/usr/local/emhttp/plugins/nix/Nix.page";
    let nix_launcher_file = "/usr/local/emhttp/plugins/nix/NixLauncher.page";
    if std::path::Path::new(nix_page_file).exists() {
        if let Ok(content) = std::fs::read_to_string(nix_page_file) {
            let updated_content = if show_in_nav == "yes" {
                let launcher_content = "Menu=\"Utilities\"\nTitle=\"Nix\"\nIcon=\"nix.png\"\n---\n<script>window.location.href = '/Settings/Nix';</script>\n";
                let _ = std::fs::write(nix_launcher_file, launcher_content);
                content.lines().map(|line| {
                    if line.starts_with("Menu=") { "Menu=\"Tasks:95\"".to_string() } else { line.to_string() }
                }).collect::<Vec<String>>().join("\n")
            } else {
                let _ = std::fs::remove_file(nix_launcher_file);
                content.lines().map(|line| {
                    if line.starts_with("Menu=") { "Menu=\"Utilities\"".to_string() } else { line.to_string() }
                }).collect::<Vec<String>>().join("\n")
            };
            let _ = std::fs::write(nix_page_file, updated_content);
        }
    }

    // Start daemon environment back up if we migrated the store path
    if migration_performed {
        let _ = std::process::Command::new("/usr/local/emhttp/plugins/nix/event/disks_mounted").output();
    }
    println!("Settings saved successfully.");
}
