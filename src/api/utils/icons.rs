use crate::unraid::PROCESS_COMPOSE_CONFIG;

pub fn get_service_icon_path(name: &str) -> Option<String> {
    let config = crate::config::load_config(PROCESS_COMPOSE_CONFIG).ok()?;
    let p = config.processes.get(name)?;
    let command = &p.command;

    if let Some(idx) = command.find("/nix/store/") {
        let sub = &command[idx..];
        let parts: Vec<&str> = sub.split('/').collect();
        if parts.len() >= 4 {
            let candidate = format!("/{}/{}/{}", parts[1], parts[2], parts[3]);
            if !candidate.contains("-bash-") && !candidate.contains("-bash-interactive-") && candidate.contains(name) {
                let path = std::path::Path::new(&candidate);
                if path.exists() {
                    if let Some(icon) = find_image_in_dir(path, path) {
                        return Some(icon);
                    }
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("/nix/store") {
        let target_pattern = format!("-{}", name.to_lowercase());
        let mut candidates = Vec::new();

        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy().to_lowercase();
            if !filename_str.ends_with(".drv") && filename_str.contains(&target_pattern) {
                candidates.push(entry.path());
            }
        }

        candidates.sort_by_key(|path| {
            let name_str = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let has_extra = name_str.contains("-web") || name_str.contains("-ffmpeg") || name_str.contains("-data") || name_str.contains("-bin") || name_str.contains("-lib");
            if has_extra { 1 } else { 0 }
        });

        for cand_path in candidates {
            if cand_path.exists() {
                if let Some(icon) = find_image_in_dir(&cand_path, &cand_path) {
                    return Some(icon);
                }
            }
        }
    }

    None
}

fn find_image_in_dir(dir: &std::path::Path, root: &std::path::Path) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(rel_path) = path.strip_prefix(root) {
                if rel_path == std::path::Path::new("bin") ||
                   rel_path == std::path::Path::new("man") ||
                   rel_path == std::path::Path::new("nix-support") ||
                   rel_path == std::path::Path::new("lib64") {
                    continue;
                }
            }
            subdirs.push(path);
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "svg" || ext_lower == "png" || ext_lower == "ico" {
                    candidates.push(path);
                }
            }
        }
    }

    for c in &candidates {
        if let Some(filename) = c.file_name().and_then(|f| f.to_str()) {
            let name_lower = filename.to_lowercase();
            if name_lower == "logo.svg" || name_lower == "logo.png" || name_lower.contains("jellyfin.svg") {
                return Some(c.to_string_lossy().to_string());
            }
        }
    }

    for c in &candidates {
        if let Some(filename) = c.file_name().and_then(|f| f.to_str()) {
            let name_lower = filename.to_lowercase();
            if name_lower.contains("logo") || name_lower.contains("icon") || name_lower.contains("favicon") {
                return Some(c.to_string_lossy().to_string());
            }
        }
    }

    for subdir in subdirs {
        if let Some(img) = find_image_in_dir(&subdir, root) {
            return Some(img);
        }
    }
    None
}
