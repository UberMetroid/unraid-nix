pub mod args;
pub mod gpus;
pub mod logs;
pub mod metadata;
pub mod render;
pub mod sandbox_check;
pub mod service;
pub mod service_install;
pub mod settings;
pub mod store;
pub mod stream_install;
pub mod supervisor;

use clap::Parser;

pub fn print_usage() {
    let mut cmd = <args::Cli as clap::CommandFactory>::command();
    let _ = cmd.print_help();
}

pub fn run(args_list: Vec<String>) {
    let cli = match args::Cli::try_parse_from(&args_list) {
        Ok(c) => c,
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Invalid CLI args: {e}"));
            std::process::exit(1);
        }
    };

    match cli.command {
        args::Commands::SetupStore {
            ref persistent_path,
        } => {
            crate::store::log_event(
                "INFO",
                &format!("Dispatch: setup-store (persistent_path={persistent_path})"),
            );
            store::setup_store(persistent_path);
        }
        args::Commands::TeardownStore => {
            crate::store::log_event("INFO", "Dispatch: teardown-store");
            store::teardown_store();
        }
        args::Commands::SyncTemplates => {
            crate::store::log_event("INFO", "Dispatch: sync-templates");
            store::sync_templates();
        }
        args::Commands::SandboxCheck { apply_fallback } => {
            sandbox_check::sandbox_check(apply_fallback);
        }
        args::Commands::Render { target } => {
            render::render(target);
        }
        args::Commands::Service { action, name } => {
            crate::store::log_event(
                "INFO",
                &format!("Dispatch: service action='{action}' name='{name}'"),
            );
            service::service_action(&action, &name);
        }
        args::Commands::Autostart { name, toggle } => {
            crate::store::log_event(
                "INFO",
                &format!("Dispatch: autostart name='{name}' toggle='{toggle}'"),
            );
            service::autostart(&name, &toggle);
        }
        args::Commands::RemoveService { name } => {
            crate::store::log_event("INFO", &format!("Dispatch: remove-service name='{name}'"));
            service::remove_service(&name);
        }
        args::Commands::Install { package } => {
            crate::store::log_event("INFO", &format!("Dispatch: install package='{package}'"));
            service::install(&package);
        }
        args::Commands::Sandbox(sandbox_args) => {
            service::sandbox_cmd(&sandbox_args);
        }
        args::Commands::Preset {
            name,
            appdata,
            media,
            puid,
            pgid,
            gpu,
            extra_binds,
            port,
            bind_address,
        } => {
            service::preset_cmd(
                &name,
                &appdata,
                &media,
                puid,
                pgid,
                &gpu,
                extra_binds.as_deref(),
                port.as_deref(),
                bind_address.as_deref(),
            );
        }
        args::Commands::AddService {
            name,
            cmd,
            restart_policy,
        } => {
            crate::store::log_event(
                "INFO",
                &format!(
                    "Dispatch: add-service name='{name}' restart_policy={}",
                    restart_policy.as_deref().unwrap_or("default")
                ),
            );
            service::add_service(&name, &cmd, restart_policy.as_deref());
        }
        args::Commands::InstallService(install_args) => {
            crate::store::log_event(
                "INFO",
                &format!("Dispatch: install-service uri='{}'", install_args.uri),
            );
            service_install::install_service(&install_args);
        }
        args::Commands::ViewLogs { name } => {
            logs::view_logs(&name);
        }
        args::Commands::SaveSettings(settings_args) => {
            let store_path = settings_args.store_path.clone().unwrap_or_default();
            let autostart = settings_args
                .autostart
                .clone()
                .unwrap_or_else(|| "yes".to_string());
            let enable_sandbox = settings_args
                .enable_sandbox
                .clone()
                .unwrap_or_else(|| "yes".to_string());
            crate::store::log_event(
                "INFO",
                &format!(
                    "Dispatch: save-settings store_path='{store_path}' autostart='{autostart}' sandbox='{enable_sandbox}'"
                ),
            );
            settings::save_settings(&settings_args);
        }
        args::Commands::GetMetadata { name } => {
            metadata::get_metadata(&name);
        }
        args::Commands::DetectGpus => {
            gpus::detect_gpus();
        }
        args::Commands::SetupGpus => {
            crate::store::log_event("INFO", "Dispatch: setup-gpus (NVIDIA/CUDA symlinks)");
            gpus::setup_gpu_driver_symlinks();
        }
        args::Commands::StreamInstall(stream_args) => {
            stream_install::stream_install(&stream_args);
        }
        args::Commands::GetIcon { name } => match crate::api::utils::get_service_icon_path(&name) {
            Some(path) => println!("{path}"),
            None => println!(),
        },
        args::Commands::DaemonStatus => {
            let status = std::process::Command::new("/etc/rc.d/rc.nix-daemon")
                .arg("status")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if let Ok(exit_status) = status {
                if exit_status.success() {
                    println!("running");
                    std::process::exit(0);
                }
            }
            println!("stopped");
            std::process::exit(1);
        }
    }
}
