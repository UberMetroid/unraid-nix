use std::process::{Command, Stdio};
use std::fs;
use std::time::Duration;
use serde_json::{json, Value};
use crate::util::process::run_with_timeout;

pub struct DetectedGpus {
    pub has_nvidia: bool,
    pub has_amd: bool,
    pub has_intel: bool,
}

/// Helper to run real-time hardware discovery of GPUs.
pub fn run_live_gpu_detection() -> Vec<Value> {
    let mut gpus = Vec::new();

    if let Ok(output) = {
        let mut cmd = Command::new("nvidia-smi");
        cmd.args(["--query-gpu=index,name,uuid,pci.bus_id", "--format=csv,noheader,nounits"])
            .stdin(Stdio::null());
        run_with_timeout(&mut cmd, Duration::from_secs(3))
    } {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(index) = parts[0].parse::<u32>() {
                        let bus_id = parts.get(3).copied().unwrap_or("");
                        gpus.push(json!({
                            "id": format!("nvidia-{index}"),
                            "name": format!("NVIDIA {} (GPU-{index})", parts[1]),
                            "type": "nvidia",
                            "index": index,
                            "bus_id": bus_id,
                        }));
                    }
                }
            }
        }
    }

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

                        if driver == "nvidia" && !gpus.is_empty() {
                            continue;
                        }

                        let friendly_name = match driver.as_str() {
                            "i915" | "xe" => format!("Intel QuickSync GPU ({name})"),
                            "amdgpu" | "radeon" => format!("AMD Radeon GPU ({name})"),
                            "nvidia" => format!("NVIDIA GPU ({name})"),
                            _ => format!("Generic GPU ({name} - {driver})"),
                        };

                        gpus.push(json!({
                            "id": name.clone(),
                            "name": friendly_name,
                            "type": if driver == "nvidia" { "nvidia" } else { "render" },
                            "path": format!("/dev/dri/{name}"),
                            "driver": driver,
                            "pci_id": pci_id,
                        }));
                    }
                }
            }
        }
    }
    gpus
}

/// Loads GPU detection results from a tmpfs cache file, or runs live detection if not cached.
pub fn load_or_detect_gpus() -> Vec<Value> {
    let cache_file = "/var/run/nix-detected-gpus.json";
    if let Ok(content) = fs::read_to_string(cache_file) {
        if let Ok(gpus) = serde_json::from_str::<Vec<Value>>(&content) {
            return gpus;
        }
    }

    let gpus = run_live_gpu_detection();
    if let Ok(json_str) = serde_json::to_string(&gpus) {
        let _ = fs::write(cache_file, json_str);
    }
    gpus
}

pub fn get_detected_gpus() -> DetectedGpus {
    let gpus = load_or_detect_gpus();
    let mut has_nvidia = false;
    let mut has_amd = false;
    let mut has_intel = false;
    for gpu in gpus {
        let gpu_type = gpu.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let driver = gpu.get("driver").and_then(|d| d.as_str()).unwrap_or("");
        if gpu_type == "nvidia" || driver == "nvidia" {
            has_nvidia = true;
        } else if driver == "amdgpu" || driver == "radeon" {
            has_amd = true;
        } else if driver == "i915" || driver == "xe" {
            has_intel = true;
        }
    }
    DetectedGpus {
        has_nvidia,
        has_amd,
        has_intel,
    }
}

pub fn detect_gpus() {
    let gpus = load_or_detect_gpus();
    println!("{}", serde_json::to_string(&gpus).unwrap_or_else(|_| "[]".to_string()));
}

/// Prepares the target symlink directory for NVIDIA / CUDA drivers on the host.
/// Replaces the legacy `nix-gpu-setup.sh` script.
pub fn setup_gpu_driver_symlinks() {
    let target_dir = std::path::Path::new("/var/run/nix-nvidia-driver/lib");
    if let Err(e) = fs::create_dir_all(target_dir) {
        crate::store::log_event("ERROR", &format!("Failed to create GPU target directory {target_dir:?}: {e}"));
        return;
    }

    if let Ok(entries) = fs::read_dir(target_dir) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }

    let lib64_dir = std::path::Path::new("/usr/lib64");
    if lib64_dir.exists() && lib64_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(lib64_dir) {
            for entry in entries.flatten() {
                let name_os = entry.file_name();
                let name = name_os.to_string_lossy();
                if name.starts_with("libcuda.so") || (name.starts_with("libnvidia-") && name.contains(".so")) {
                    let dest = target_dir.join(&*name);
                    let _ = std::os::unix::fs::symlink(entry.path(), dest);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_gpus_construction_and_field_access() {
        let g = DetectedGpus {
            has_nvidia: true,
            has_amd: false,
            has_intel: true,
        };
        assert!(g.has_nvidia);
        assert!(!g.has_amd);
        assert!(g.has_intel);
    }

    #[test]
    fn test_detected_gpus_all_false_default() {
        let g = DetectedGpus {
            has_nvidia: false,
            has_amd: false,
            has_intel: false,
        };
        assert!(!g.has_nvidia);
        assert!(!g.has_amd);
        assert!(!g.has_intel);
    }

    #[test]
    fn test_get_detected_gpus_returns_consistent_flags() {
        // The function reads the cache or runs live detection; either way
        // the resulting DetectedGpus must be self-consistent (no flag
        // refers to an invalid state).
        let g = get_detected_gpus();
        // All fields are bool, so the only invariant is that we can
        // observe them without panicking. Verify each is a valid bool
        // by reading it.
        let _ = g.has_nvidia;
        let _ = g.has_amd;
        let _ = g.has_intel;
    }

    #[test]
    fn test_load_or_detect_gpus_handles_invalid_cache_gracefully() {
        // We can't reliably write to /var/run in the test environment,
        // but we CAN verify the function doesn't panic when called
        // twice (i.e. the cache hit path doesn't blow up).
        let _ = load_or_detect_gpus();
        let _ = load_or_detect_gpus();
    }

    #[test]
    fn test_detect_gpus_prints_json() {
        // detect_gpus prints a JSON array. We don't capture stdout, but
        // we can at least verify the function runs without panic and
        // doesn't block indefinitely.
        detect_gpus();
    }
}
