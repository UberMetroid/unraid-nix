pub mod store;
pub mod service;
pub mod settings;
pub mod render;
pub mod logs;
pub mod service_install;
pub mod supervisor;
pub mod metadata;
pub mod gpus;

pub fn print_usage() {
    println!("Usage: nix-helper <subcommand> [args]");
    println!("Subcommands:");
    println!("  setup-store <persistent_path>          Initializes and bind-mounts /nix");
    println!("  teardown-store                         Stops services and cleanly unmounts /nix");
    println!("  render services                        Outputs HTML services dashboard table");
    println!("  render search <query>                  Outputs HTML packages search table");
    println!("  render presets                         Outputs HTML preset services store grid");
    println!("  render dashboard                       Outputs HTML dashboard widget structure");
    println!("  render dashboard-rows                  Outputs HTML dashboard widget service rows");
    println!("  service <start|stop|restart> <name>    Sends action triggers to process-compose");
    println!("  autostart <name> <on|off>              Toggles the autostart setting for a service");
    println!("  remove-service <name>                  Deletes a service definition from the config");
    println!("  install <package>                      Installs package to CLI profile");
    println!("  sandbox <options>                      Helper command to print bubblewrap script");
    println!("  preset <args...>                       Helper command to print preset bubblewrap script");
    println!("  add-service <name> <cmd> [restart]     Adds a service to process-compose configuration");
    println!("  install-service <options>              Installs a service, creates folders/metadata, and adds it");
    println!("  view-logs <name>                       Outputs formatted service console logs");
    println!("  save-settings <options>                Saves Nix plugin settings and manages migration");
    println!("  get-metadata <name>                    Outputs JSON service metadata");
    println!("  detect-gpus                            Outputs JSON list of detected host GPUs");
    println!("  get-icon <name>                        Outputs the absolute path of a service logo in the Nix store");
}

pub fn run(args: Vec<String>) {
    let subcommand = args[1].as_str();
    match subcommand {
        "setup-store" => store::setup_store(&args),
        "teardown-store" => store::teardown_store(),
        "render" => render::render(&args),
        "service" => service::service_action(&args),
        "autostart" => service::autostart(&args),
        "remove-service" => service::remove_service(&args),
        "install" => service::install(&args),
        "sandbox" => service::sandbox_cmd(&args),
        "preset" => service::preset_cmd(&args),
        "add-service" => service::add_service(&args),
        "install-service" => service_install::install_service(&args),
        "view-logs" => logs::view_logs(&args),
        "save-settings" => settings::save_settings(&args),
        "get-metadata" => metadata::get_metadata(&args),
        "detect-gpus" => gpus::detect_gpus(&args),
        "get-icon" => {
            let name = if args.len() >= 3 { &args[2] } else { "" };
            if let Some(path) = crate::api::utils::get_service_icon_path(name) {
                println!("{}", path);
            } else {
                println!("");
            }
        }
        _ => {
            print_usage();
        }
    }
}
