/// Nix Sandbox Builder Module
///
/// This module constructs the bubblewrap (bwrap) sandbox commands
/// to isolate Nix processes, control host filesystem access (shares),
/// apply PUID/PGID spoofing, and optionally bind GPU rendering devices.
use std::path::Path;

/// Configuration options for building the bubblewrap sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub name: String,
    pub appdata_path: String,
    pub media_path: Option<String>,
    pub puid: u32,
    pub pgid: u32,
    pub enable_gpu: bool,
    pub inner_command: String,
}

/// Generates the full bubblewrap execution command string.
///
/// Builds a clean filesystem namespace, binding only the required host directories
/// and remapping paths (e.g. appdata -> /config) to match Docker user expectations.
pub fn build_bwrap_command(config: &SandboxConfig) -> Result<String, String> {
    if config.appdata_path.trim().is_empty() {
        return Err("Appdata path must be specified for service sandboxing.".to_string());
    }

    let mut args = vec![
        "bwrap".to_string(),
        "--ro-bind /usr /usr".to_string(),
        "--ro-bind /lib /lib".to_string(),
        "--ro-bind /bin /bin".to_string(),
        "--ro-bind /nix /nix".to_string(),
        // Network configurations
        "--ro-bind /etc/resolv.conf /etc/resolv.conf".to_string(),
        "--ro-bind /etc/hosts /etc/hosts".to_string(),
        "--ro-bind /etc/ssl /etc/ssl".to_string(),
        // Vital system tables
        "--ro-bind /etc/passwd /etc/passwd".to_string(),
        "--ro-bind /etc/group /etc/group".to_string(),
        // Devices and process table
        "--dev /dev".to_string(),
        "--proc /proc".to_string(),
        // PUID / PGID Spoofing
        format!("--uid {}", config.puid),
        format!("--gid {}", config.pgid),
    ];

    // Check for lib64 which is present on 64-bit Slackware/Unraid
    if Path::new("/lib64").exists() {
        args.insert(3, "--ro-bind /lib64 /lib64".to_string());
    }

    // Bind Appdata share to /config
    args.push(format!("--bind {} /config", config.appdata_path));

    // Optionally bind Media share to /media
    if let Some(ref media) = config.media_path {
        if !media.trim().is_empty() {
            args.push(format!("--bind {} /media", media));
        }
    }

    // Expose GPU render nodes if hardware acceleration is enabled
    if config.enable_gpu {
        if Path::new("/dev/dri").exists() {
            args.push("--dev-bind /dev/dri /dev/dri".to_string());
        }
        // Nvidia support bindings if they exist on the host
        if Path::new("/dev/nvidia0").exists() {
            args.push("--dev-bind /dev/nvidiactl /dev/nvidiactl".to_string());
            args.push("--dev-bind /dev/nvidia0 /dev/nvidia0".to_string());
            if Path::new("/usr/lib64/nvidia").exists() {
                args.push("--ro-bind /usr/lib64/nvidia /usr/lib64/nvidia".to_string());
            }
        }
    }

    // Default working directory inside the sandbox
    args.push("--chdir /".to_string());

    // Execute the actual command via bash wrapper inside the sandbox
    args.push(format!("sh -c \"{}\"", config.inner_command.replace("\"", "\\\"")));

    Ok(args.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_bwrap_command_basic() {
        let config = SandboxConfig {
            name: "test-app".to_string(),
            appdata_path: "/mnt/cache/appdata/test-app".to_string(),
            media_path: Some("/mnt/user/media".to_string()),
            puid: 99,
            pgid: 100,
            enable_gpu: false,
            inner_command: "nix run nixpkgs#hello".to_string(),
        };

        let cmd = build_bwrap_command(&config).unwrap();
        assert!(cmd.starts_with("bwrap "));
        assert!(cmd.contains("--uid 99"));
        assert!(cmd.contains("--gid 100"));
        assert!(cmd.contains("--bind /mnt/cache/appdata/test-app /config"));
        assert!(cmd.contains("--bind /mnt/user/media /media"));
        assert!(cmd.contains("sh -c \"nix run nixpkgs#hello\""));
    }

    #[test]
    fn test_build_bwrap_command_gpu() {
        let config = SandboxConfig {
            name: "jellyfin".to_string(),
            appdata_path: "/mnt/cache/appdata/jellyfin".to_string(),
            media_path: None,
            puid: 99,
            pgid: 100,
            enable_gpu: true,
            inner_command: "jellyfin".to_string(),
        };

        let cmd = build_bwrap_command(&config).unwrap();
        // Since GPU is requested, check that it generates standard binds
        assert!(cmd.contains("--chdir /"));
        // If /dev/dri exists on the test runner, it will be mapped. We can't guarantee that in test,
        // but we verify the command still successfully builds without crashing.
    }

    #[test]
    fn test_build_bwrap_command_missing_appdata() {
        let config = SandboxConfig {
            name: "test-app".to_string(),
            appdata_path: "".to_string(),
            media_path: None,
            puid: 99,
            pgid: 100,
            enable_gpu: false,
            inner_command: "run".to_string(),
        };

        let err = build_bwrap_command(&config);
        assert!(err.is_err());
    }
}
