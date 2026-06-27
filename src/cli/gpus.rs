use std::process::Command;
use std::fs;
use serde_json::json;

/// Scans the host system to identify all available GPU devices.
/// Queries nvidia-smi for NVIDIA GPUs and scans /sys/class/drm for Intel/AMD DRM render nodes.
/// Returns a JSON-formatted list of detected GPUs.
pub fn detect_gpus(_args: &[String]) {
    let mut gpus = Vec::new();

    // 1. Try running nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu=index,name,uuid,pci.bus_id", "--format=csv,noheader,nounits"])
        .output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(index) = parts[0].parse::<u32>() {
                        gpus.push(json!({
                            "id": format!("nvidia-{}", index),
                            "name": format!("NVIDIA {} (GPU-{})", parts[1], index),
                            "type": "nvidia",
                            "index": index,
                            "bus_id": parts.get(3).unwrap_or(&"").to_string(),
                        }));
                    }
                }
            }
        }
    }

    // 2. Scan /sys/class/drm for other GPUs
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("renderD") {
                let uevent_path = entry.path().join("device/uevent");
                if uevent_path.exists() {
                    if let Ok(content) = fs::read_to_string(&uevent_path) {
                        let mut driver = String::new();
                        let mut pci_id = String::new();
                        for line in content.lines() {
                            if line.starts_with("DRIVER=") {
                                driver = line.trim_start_matches("DRIVER=").to_string();
                            } else if line.starts_with("PCI_ID=") {
                                pci_id = line.trim_start_matches("PCI_ID=").to_string();
                            }
                        }
                        
                        // Skip nvidia here to avoid duplicate toggles, unless nvidia-smi failed and no nvidia GPUs were added
                        if driver == "nvidia" && !gpus.is_empty() {
                            continue;
                        }

                        let friendly_name = match driver.as_str() {
                            "i915" | "xe" => format!("Intel QuickSync GPU ({})", name),
                            "amdgpu" | "radeon" => format!("AMD Radeon GPU ({})", name),
                            "nvidia" => format!("NVIDIA GPU ({})", name),
                            _ => format!("Generic GPU ({} - {})", name, driver),
                        };

                        gpus.push(json!({
                            "id": name.clone(),
                            "name": friendly_name,
                            "type": if driver == "nvidia" { "nvidia" } else { "render" },
                            "path": format!("/dev/dri/{}", name),
                            "driver": driver,
                            "pci_id": pci_id,
                        }));
                    }
                }
            }
        }
    }

    println!("{}", serde_json::to_string(&gpus).unwrap_or_else(|_| "[]".to_string()));
}
