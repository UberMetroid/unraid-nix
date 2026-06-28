pub mod tail;

use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

/// Custom streamer for real-time installer output.
/// Replaces the legacy `web/stream.php` capture loops.
pub fn stream_install(args: &[String]) {
    let mut action = "";
    let mut install_type = "";
    let mut uri = "";
    let mut appdata = "";
    let mut media = "";
    let mut puid = "99";
    let mut pgid = "100";
    let mut gpu = "0";
    let mut gpus = "";
    let mut extra_binds = "";
    let mut port = "";
    let mut bind_address = "";
    let mut env_vars = "";
    let mut compile_locally = false;
    let mut command_override = "";
    let mut network_isolation = "0";

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--action" => { if i + 1 < args.len() { action = &args[i+1]; } i += 2; }
            "--type" => { if i + 1 < args.len() { install_type = &args[i+1]; } i += 2; }
            "--uri" => { if i + 1 < args.len() { uri = &args[i+1]; } i += 2; }
            "--appdata" => { if i + 1 < args.len() { appdata = &args[i+1]; } i += 2; }
            "--media" => { if i + 1 < args.len() { media = &args[i+1]; } i += 2; }
            "--puid" => { if i + 1 < args.len() { puid = &args[i+1]; } i += 2; }
            "--pgid" => { if i + 1 < args.len() { pgid = &args[i+1]; } i += 2; }
            "--gpu" => { if i + 1 < args.len() { gpu = &args[i+1]; } i += 2; }
            "--gpus" => { if i + 1 < args.len() { gpus = &args[i+1]; } i += 2; }
            "--extra-binds" => { if i + 1 < args.len() { extra_binds = &args[i+1]; } i += 2; }
            "--port" => { if i + 1 < args.len() { port = &args[i+1]; } i += 2; }
            "--bind-address" => { if i + 1 < args.len() { bind_address = &args[i+1]; } i += 2; }
            "--env-vars" => { if i + 1 < args.len() { env_vars = &args[i+1]; } i += 2; }
            "--network-isolation" => { if i + 1 < args.len() { network_isolation = &args[i+1]; } i += 2; }
            "--compile-locally" => { compile_locally = true; i += 1; }
            "--command-override" => { if i + 1 < args.len() { command_override = &args[i+1]; } i += 2; }
            _ => { i += 1; }
        }
    }

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
    svc.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect()
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
