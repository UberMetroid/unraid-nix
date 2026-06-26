/// Nix Helper CLI Entrypoint
///
/// This is the main router for the unraid-nix compiled Rust binary.
/// It parses arguments from standard environment variables, handles
/// routing to respective modules, and outputs JSON or HTML payloads.
use std::env;

mod api;
mod config;
mod process;
mod sandbox;
mod store;
mod search;

fn print_usage() {
    println!("Usage: nix-helper <subcommand> [args]");
    println!("Subcommands:");
    println!("  setup-store <persistent_path>          Initializes and bind-mounts /nix");
    println!("  teardown-store                         Stops services and cleanly unmounts /nix");
    println!("  render services                        Outputs HTML services dashboard table");
    println!("  render search <query>                  Outputs HTML packages search table");
    println!("  render dashboard                       Outputs HTML dashboard widget rows");
    println!("  service <start|stop|restart> <name>    Sends action triggers to process-compose");
    println!("  autostart <name> <on|off>              Toggles the autostart setting for a service");
    println!("  remove-service <name>                  Deletes a service definition from the config");
    println!("  install <package>                      Installs package to CLI profile");
    println!("  sandbox <options>                      Helper command to print bubblewrap script");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let subcommand = args[1].as_str();
    match subcommand {
        "setup-store" => {
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
        "teardown-store" => {
            if let Err(e) = store::unmount_nix_store() {
                eprintln!("Error unmounting store: {}", e);
                std::process::exit(1);
            }
            println!("Nix Store successfully unmounted.");
        }
        "render" => {
            if args.len() < 3 {
                print_usage();
                return;
            }
            match args[2].as_str() {
                "services" => {
                    println!("{}", api::render_services_table(29704));
                }
                "search" => {
                    let query = if args.len() >= 4 { &args[3] } else { "" };
                    println!("{}", api::render_search_results(query));
                }
                "dashboard" => {
                    println!("{}", api::render_dashboard_widget(29704));
                }
                _ => print_usage(),
            }
        }
        "service" => {
            if args.len() < 4 {
                eprintln!("Error: Missing action or service name.");
                std::process::exit(1);
            }
            let action = &args[2];
            let name = &args[3];
            if let Err(e) = process::send_service_action(29704, name, action) {
                eprintln!("Service action failed: {}", e);
                std::process::exit(1);
            }
            println!("Action {} sent to service {}.", action, name);
        }
        "install" => {
            if args.len() < 3 {
                eprintln!("Error: Missing package name.");
                std::process::exit(1);
            }
            let package = &args[2];
            if let Err(e) = search::install_package(package) {
                eprintln!("Installation failed: {}", e);
                std::process::exit(1);
            }
            println!("Successfully installed package: {}", package);
        }
        "sandbox" => {
            match parse_sandbox_args(&args) {
                Ok(cmd) => println!("{}", cmd),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "preset" => {
            if args.len() < 8 {
                eprintln!("Error: Missing arguments: preset <name> <appdata> <media> <puid> <pgid> <gpu> [extra_binds]");
                std::process::exit(1);
            }
            let name = &args[2];
            let appdata = &args[3];
            let media = if args[4] == "-" { "" } else { &args[4] };
            let puid = args[5].parse::<u32>().unwrap_or(99);
            let pgid = args[6].parse::<u32>().unwrap_or(100);
            let gpu = args[7] == "1" || args[7] == "true";
            let extra_binds = if args.len() >= 9 {
                parse_binds_string(&args[8]).unwrap_or_default()
            } else {
                Vec::new()
            };

            match config::get_service_command_preset(name, appdata, media, puid, pgid, gpu, extra_binds) {
                Ok(cmd) => println!("{}", cmd),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "add-service" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments: add-service <name> <command> [restart_policy]");
                std::process::exit(1);
            }
            let name = args[2].clone();
            let cmd = args[3].clone();
            let restart = if args.len() >= 5 { args[4].clone() } else { "always".to_string() };

            let mut cfg = match config::load_config("/boot/config/plugins/nix/process-compose.yml") {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                }
            };

            // Ensure global log configuration exists
            if cfg.log_configuration.is_none() {
                cfg.log_configuration = Some(config::LogConfiguration {
                    add_timestamp: Some(true),
                    fields_order: Some(vec![
                        "time".to_string(),
                        "level".to_string(),
                        "message".to_string(),
                    ]),
                });
            }

            let log_location = Some(format!("/var/log/nix-services/{}.log", name));
            cfg.processes.insert(name, config::ProcessDefinition {
                command: cmd,
                availability: Some(config::Availability {
                    restart,
                    backoff_seconds: Some(5),
                    max_restarts: None,
                }),
                environment: None,
                log_location,
                log_configuration: None,
            });

            if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
                eprintln!("Error saving config: {}", e);
                std::process::exit(1);
            }
            println!("Service successfully added.");
        }
        "autostart" => {
            if args.len() < 4 {
                eprintln!("Error: Missing service name or toggle value (on/off).");
                std::process::exit(1);
            }
            let name = &args[2];
            let toggle = args[3].to_lowercase();
            let restart_policy = if toggle == "on" || toggle == "true" || toggle == "1" {
                "always".to_string()
            } else {
                "no".to_string()
            };

            let mut cfg = match config::load_config("/boot/config/plugins/nix/process-compose.yml") {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                }
            };

            if let Some(p) = cfg.processes.get_mut(name) {
                if let Some(ref mut a) = p.availability {
                    a.restart = restart_policy;
                } else {
                    p.availability = Some(config::Availability {
                        restart: restart_policy,
                        backoff_seconds: Some(5),
                        max_restarts: None,
                    });
                }

                if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
                    eprintln!("Error saving config: {}", e);
                    std::process::exit(1);
                }

                // Trigger process-compose project update to apply changes dynamically without dropping running services
                let reload_output = std::process::Command::new("sh")
                    .args(&["-c", ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml 2>&1"])
                    .output();
                
                if let Ok(out) = reload_output {
                    if !out.status.success() {
                        let err_msg = String::from_utf8_lossy(&out.stderr);
                        let out_msg = String::from_utf8_lossy(&out.stdout);
                        eprintln!("Warning: process-compose project update returned non-zero status. Output: {} {}", out_msg, err_msg);
                    }
                }
                println!("Autostart updated successfully.");
            } else {
                eprintln!("Error: Service {} not found in configuration.", name);
                std::process::exit(1);
            }
        }
        "remove-service" => {
            if args.len() < 3 {
                eprintln!("Error: Missing service name.");
                std::process::exit(1);
            }
            let name = &args[2];

            let mut cfg = match config::load_config("/boot/config/plugins/nix/process-compose.yml") {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                }
            };

            if cfg.processes.remove(name).is_some() {
                if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
                    eprintln!("Error saving config: {}", e);
                    std::process::exit(1);
                }

                // Stop the process if running (ignore errors if already stopped)
                let _ = process::send_service_action(29704, name, "stop");

                // Trigger process-compose project update to apply changes dynamically and remove the process
                let reload_output = std::process::Command::new("sh")
                    .args(&["-c", ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml 2>&1"])
                    .output();
                
                if let Ok(out) = reload_output {
                    if !out.status.success() {
                        let err_msg = String::from_utf8_lossy(&out.stderr);
                        let out_msg = String::from_utf8_lossy(&out.stdout);
                        eprintln!("Warning: process-compose project update returned non-zero status. Output: {} {}", out_msg, err_msg);
                    }
                }
                println!("Service {} successfully removed.", name);
            } else {
                eprintln!("Error: Service {} not found in configuration.", name);
                std::process::exit(1);
            }
        }
        _ => {
            print_usage();
        }
    }
}

/// Helper method to parse CLI sandbox arguments manually without large clap crate.
fn parse_sandbox_args(args: &[String]) -> Result<String, String> {
    let mut name = String::new();
    let mut appdata = String::new();
    let mut media = None;
    let mut puid = 99;
    let mut pgid = 100;
    let mut gpu = false;
    let mut cmd = String::new();
    let mut extra_binds = Vec::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                if i + 1 >= args.len() { return Err("Missing value for --name".to_string()); }
                name = args[i+1].clone();
                i += 2;
            }
            "--appdata" => {
                if i + 1 >= args.len() { return Err("Missing value for --appdata".to_string()); }
                appdata = args[i+1].clone();
                i += 2;
            }
            "--media" => {
                if i + 1 >= args.len() { return Err("Missing value for --media".to_string()); }
                let val = args[i+1].clone();
                media = if val.trim().is_empty() || val == "-" { None } else { Some(val) };
                i += 2;
            }
            "--puid" => {
                if i + 1 >= args.len() { return Err("Missing value for --puid".to_string()); }
                puid = args[i+1].parse::<u32>().map_err(|_| "Invalid PUID")?;
                i += 2;
            }
            "--pgid" => {
                if i + 1 >= args.len() { return Err("Missing value for --pgid".to_string()); }
                pgid = args[i+1].parse::<u32>().map_err(|_| "Invalid PGID")?;
                i += 2;
            }
            "--gpu" => {
                gpu = true;
                i += 1;
            }
            "--cmd" => {
                if i + 1 >= args.len() { return Err("Missing value for --cmd".to_string()); }
                cmd = args[i+1].clone();
                i += 2;
            }
            "--extra-binds" => {
                if i + 1 >= args.len() { return Err("Missing value for --extra-binds".to_string()); }
                extra_binds = parse_binds_string(&args[i+1])?;
                i += 2;
            }
            _ => return Err(format!("Unknown sandbox flag: {}", args[i])),
        }
    }

    sandbox::build_bwrap_command(&sandbox::SandboxConfig {
        name,
        appdata_path: appdata,
        media_path: media,
        puid,
        pgid,
        enable_gpu: gpu,
        inner_command: cmd,
        extra_binds,
    })
}

fn parse_binds_string(s: &str) -> Result<Vec<(String, String)>, String> {
    let mut binds = Vec::new();
    if s.trim().is_empty() {
        return Ok(binds);
    }
    for part in s.split(',') {
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() != 2 {
            return Err(format!("Invalid extra bind format: '{}'. Expected 'host:sandbox'.", part));
        }
        binds.push((subparts[0].to_string(), subparts[1].to_string()));
    }
    Ok(binds)
}
