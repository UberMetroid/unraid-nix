use std::process::Command;
use std::fs;
use super::status::GpuStat;

pub fn get_gpu_active_services() -> std::collections::HashSet<String> {
    let mut active_services = std::collections::HashSet::new();

    if !crate::cli::gpus::get_detected_gpus().has_nvidia {
        return active_services;
    }

    let output = Command::new("nvidia-smi")
        .args(["--query-compute-apps=pid", "--format=csv,noheader,nounits"])
        .stdin(std::process::Stdio::null())
        .output();
 
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let pid_str = line.trim();
                if pid_str.is_empty() || pid_str == "No running processes found" {
                    continue;
                }
                if let Ok(pid) = pid_str.parse::<i32>() {
                    let root_link = format!("/proc/{}/root", pid);
                    if let Ok(target) = std::fs::read_link(&root_link) {
                        let target_str = target.to_string_lossy();
                        if let Some(pos) = target_str.find("nix-chroot-") {
                            let start = pos + "nix-chroot-".len();
                            let service_name = &target_str[start..];
                            if !service_name.is_empty() {
                                  active_services.insert(service_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    active_services
}

pub fn get_proc_io(pid: i32) -> Option<(u64, u64)> {
    let io_file = format!("/proc/{}/io", pid);
    if let Ok(content) = fs::read_to_string(io_file) {
        let mut rchar = None;
        let mut wchar = None;
        for line in content.lines() {
            if line.starts_with("rchar:") {
                rchar = line.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            } else if line.starts_with("wchar:") {
                wchar = line.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            }
        }
        if let (Some(rc), Some(wc)) = (rchar, wchar) {
            return Some((rc, wc));
        }
    }
    None
}

pub fn get_descendant_pids(parent_pid: i32) -> Vec<i32> {
    let mut ppid_map: std::collections::HashMap<i32, Vec<i32>> = std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(child_pid) = name.parse::<i32>() {
                        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", child_pid)) {
                            if let Some(pos) = stat.rfind(')') {
                                let fields_after_name = &stat[pos+1..];
                                let mut parts = fields_after_name.split_whitespace();
                                let _state = parts.next();
                                if let Some(ppid_str) = parts.next() {
                                    if let Ok(ppid) = ppid_str.parse::<i32>() {
                                        ppid_map.entry(ppid).or_default().push(child_pid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Use a HashSet for O(1) membership tests during BFS instead of
    // pids.contains(&c) which is O(n) — keeps the walk O(n) overall even
    // on hosts with thousands of processes.
    let mut pids = vec![parent_pid];
    let mut seen: std::collections::HashSet<i32> = std::collections::HashSet::new();
    seen.insert(parent_pid);
    let mut i = 0;
    while i < pids.len() {
        let p = pids[i];
        if let Some(children) = ppid_map.get(&p) {
            for &c in children {
                if seen.insert(c) {
                    pids.push(c);
                }
            }
        }
        i += 1;
    }
    pids
}

pub fn get_nvidia_pmon_stats() -> std::collections::HashMap<i32, Vec<(i32, GpuStat)>> {
    let mut stats = std::collections::HashMap::new();

    if !crate::cli::gpus::get_detected_gpus().has_nvidia {
        return stats;
    }

    let output = Command::new("nvidia-smi")
        .args(["pmon", "-c", "1"])
        .stdin(std::process::Stdio::null())
        .output();
    if let Ok(out) = output {
        if let Ok(stdout_str) = String::from_utf8(out.stdout) {
            for line in stdout_str.lines() {
                if line.starts_with('#') || line.trim().is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let gpu_str = parts[0];
                    let pid_str = parts[1];
                    let sm_str = parts[3];
                    let mem_str = parts[4];
                    if let (Ok(gpu), Ok(pid)) = (gpu_str.parse::<i32>(), pid_str.parse::<i32>()) {
                        let sm = sm_str.parse::<i32>().unwrap_or(0);
                        let mem = mem_str.parse::<i32>().unwrap_or(0);
                        stats.entry(pid).or_insert_with(Vec::new).push((gpu, GpuStat { sm, mem }));
                      }
                  }
              }
          }
      }
      stats
  }
