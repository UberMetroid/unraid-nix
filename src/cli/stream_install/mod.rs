pub mod tail;

use std::process::{Command, Stdio, exit};
use std::thread::sleep;
use std::time::Duration;

/// Custom streamer for real-time installer output.
/// Replaces the legacy `web/stream.php` capture loops.
pub fn stream_install(args: &crate::cli::args::StreamInstallArgs) {
    let action = &args.action;
    let install_type = args.r#type.as_deref().unwrap_or("");
    let uri = &args.uri;
    let appdata = args.appdata.as_deref().unwrap_or("");
    let media = args.media.as_deref().unwrap_or("");
    let puid = args.puid.as_deref().unwrap_or("99");
    let pgid = args.pgid.as_deref().unwrap_or("100");
    let gpu = args.gpu.as_deref().unwrap_or("0");
    let gpus = args.gpus.as_deref().unwrap_or("");
    let extra_binds = args.extra_binds.as_deref().unwrap_or("");
    let port = args.port.as_deref().unwrap_or("");
    let bind_address = args.bind_address.as_deref().unwrap_or("");
    let env_vars = args.env_vars.as_deref().unwrap_or("");
    let compile_locally = args.compile_locally;
    let command_override = args.command_override.as_deref().unwrap_or("");
    let network_isolation = args.network_isolation.as_deref().unwrap_or("0");

    let mut cmd_args = Vec::new();
    let mut is_service = false;
    let mut timeout_limit = 45;

    if action == "install-cli" {
        cmd_args.push("install".to_string());
        cmd_args.push(uri.to_string());
    } else if action == "install-custom" {
        if install_type == "cli" {
            cmd_args.push("install".to_string());
            cmd_args.push(uri.to_string());
        } else if install_type == "service" {
            is_service = true;
            cmd_args.push("install-service".to_string());
            cmd_args.push("--uri".to_string()); cmd_args.push(uri.to_string());
            cmd_args.push("--appdata".to_string()); cmd_args.push(appdata.to_string());
            cmd_args.push("--media".to_string()); cmd_args.push(media.to_string());
            cmd_args.push("--puid".to_string()); cmd_args.push(puid.to_string());
            cmd_args.push("--pgid".to_string()); cmd_args.push(pgid.to_string());
            cmd_args.push("--gpu".to_string()); cmd_args.push(gpu.to_string());
            cmd_args.push("--gpus".to_string()); cmd_args.push(gpus.to_string());
            cmd_args.push("--extra-binds".to_string()); cmd_args.push(extra_binds.to_string());
            cmd_args.push("--port".to_string()); cmd_args.push(port.to_string());
            cmd_args.push("--bind-address".to_string()); cmd_args.push(bind_address.to_string());
            cmd_args.push("--env-vars".to_string()); cmd_args.push(env_vars.to_string());
            cmd_args.push("--network-isolation".to_string()); cmd_args.push(network_isolation.to_string());
            if !command_override.is_empty() {
                cmd_args.push("--command-override".to_string());
                cmd_args.push(command_override.to_string());
            }
            if compile_locally {
                cmd_args.push("--compile-locally".to_string());
                timeout_limit = 1800;
            }
        }
    }

    let status = Command::new("/usr/local/emhttp/plugins/nix/nix-helper")
        .args(&cmd_args)
        .env("NIX_REMOTE", "daemon")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    let mut code = match status {
        Ok(s) => s.code().unwrap_or(-1),
        Err(_) => -1,
    };

    let svc = parse_service_name(uri);

    if code == 0 {
        if is_service {
            print_step_script(1, "done", "1. Resolving Flake package & dependencies...", "Complete");
            print_step_script(2, "running", "2. Running pre-flight checks (ports & permissions)...", "Running");
            sleep(Duration::from_millis(400));
            print_step_script(2, "done", "2. Running pre-flight checks (ports & permissions)...", "Complete");
            print_step_script(3, "running", "3. Constructing sandbox jail & mounting paths...", "Running");
            sleep(Duration::from_millis(400));
            print_step_script(3, "done", "3. Constructing sandbox jail & mounting paths...", "Complete");
            print_step_script(4, "running", "4. Injecting env variables & log rotation limits...", "Running");
            sleep(Duration::from_millis(400));
            print_step_script(4, "done", "4. Injecting env variables & log rotation limits...", "Complete");
            print_step_script(5, "running", "5. Starting process supervisor & verifying liveness...", "Running");
            
            // Tail service log file and verify liveness
            match tail::tail_service_logs(&svc, timeout_limit) {
                Ok(success) => {
                    if !success { code = -1; }
                }
                Err(_) => { code = -1; }
            }
        } else {
            print_step_script(1, "done", "1. Resolving Flake package & dependencies...", "Complete");
        }
    } else {
        print_step_script(1, "failed", "1. Resolving Flake package & dependencies...", "Failed");
        println!("<script>if (document.getElementById('overall-status')) {{ document.getElementById('overall-status').innerHTML = '<i class=\"fa fa-times-circle error\"></i> Failed'; }}</script>");
    }

    if is_service {
        if code == 0 {
            crate::unraid::send_unraid_notification(
                &format!("Nix: Service '{}' Installed", svc),
                &format!("The service '{}' has been successfully installed and launched inside the Nix sandbox.", svc),
                "normal",
            );
        } else {
            crate::unraid::send_unraid_notification(
                &format!("Nix: Service '{}' Install Failed", svc),
                &format!("The installation or startup of service '{}' failed. Check the install log for details.", svc),
                "alert",
            );
        }
    } else {
        if code == 0 {
            crate::unraid::send_unraid_notification(
                "Nix: Package Installed",
                "The package/operation was completed successfully.",
                "normal",
            );
        } else {
            crate::unraid::send_unraid_notification(
                "Nix: Package Install Failed",
                "The package installation or shell execution failed.",
                "alert",
            );
        }
    }

    let report_html = if code == 0 && is_service {
        get_report_html(&svc)
    } else {
        "".to_string()
    };

    // Print final JS finish call
    println!(
        "<script>finishInstallation({}, {}, {}, {}, {});</script>",
        code,
        serde_json::to_string(action).unwrap_or_default(),
        serde_json::to_string(install_type).unwrap_or_default(),
        serde_json::to_string(&svc).unwrap_or_default(),
        serde_json::to_string(&report_html).unwrap_or_default()
    );
}

fn parse_service_name(uri: &str) -> String {
    let mut svc = uri.to_lowercase().replace("nixpkgs#", "");
    if let Some(last) = svc.split('/').last() { svc = last.to_string(); }
    if let Some(last) = svc.split(':').last() { svc = last.to_string(); }
    if let Some(last) = svc.split('#').last() { svc = last.to_string(); }
    
    if !crate::store::is_valid_service_name(&svc) {
        eprintln!("Error: Derived service name '{}' is invalid.", svc);
        exit(1);
    }
    svc
}

fn print_step_script(step: u32, status: &str, text: &str, badge: &str) {
    println!(
        "<script>if (typeof setStepStatus === 'function') {{ setStepStatus({}, '{}', '{}', '{}'); }}</script>",
        step, status, text, badge
    );
}

fn get_report_html(svc: &str) -> String {
    let output = Command::new("/usr/local/emhttp/plugins/nix/nix-helper")
        .args(["render", "report", svc])
        .stdin(Stdio::null())
        .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => "".to_string(),
    }
}
