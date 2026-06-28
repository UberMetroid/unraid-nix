use std::process::Command;
use super::config::log_event;

/// Generates the system commands to create static nixbld groups and users.
/// Enforces high UID/GID range (30000+) to prevent clashes with Unraid GUI users.
pub fn get_user_add_commands() -> Vec<String> {
    let mut cmds = vec![
        "groupadd -g 30000 nixbld 2>/dev/null || true".to_string()
    ];
    for i in 1..=32 {
        let uid = 30000 + i;
        cmds.push(format!(
            "useradd -u {} -g nixbld -G nixbld -d /var/empty -s /bin/false -c \"Nix build user {}\" nixbld{} 2>/dev/null || true",
            uid, i, i
        ));
    }
    cmds
}

/// Creates the static nixbld builder users and groups on the host.
pub fn create_builder_accounts() -> Result<(), String> {
    log_event("INFO", "Creating static nixbld builder users and group (UID/GID 30000+)...");
    for cmd in get_user_add_commands() {
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdin(std::process::Stdio::null())
            .status()
            .map_err(|e| {
                let err_msg = format!("Failed to execute builder user/group command: {}", e);
                log_event("ERROR", &err_msg);
                err_msg
            })?;
        if !status.success() {
            // Ignore failures if group/user already exists
            continue;
        }
    }
    log_event("INFO", "Nix builder accounts verified/created.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_add_commands() {
        let cmds = get_user_add_commands();
        assert_eq!(cmds.len(), 33); // 1 groupadd + 32 useradds
        assert!(cmds[0].contains("groupadd -g 30000 nixbld"));
        assert!(cmds[1].contains("useradd -u 30001 -g nixbld"));
        assert!(cmds[32].contains("useradd -u 30032 -g nixbld"));
    }
}
