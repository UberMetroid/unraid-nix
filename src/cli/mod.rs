pub mod args;
pub mod store;
pub mod service;
pub mod settings;
pub mod render;
pub mod logs;
pub mod service_install;
pub mod supervisor;
pub mod metadata;
pub mod gpus;
pub mod stream_install;

use clap::Parser;

pub fn print_usage() {
    let mut cmd = <args::Cli as clap::CommandFactory>::command();
    let _ = cmd.print_help();
}

pub fn run(args_list: Vec<String>) {
    let cli = match args::Cli::try_parse_from(&args_list) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        args::Commands::SetupStore { persistent_path } => {
            store::setup_store(&persistent_path);
        }
        args::Commands::TeardownStore => {
            store::teardown_store();
        }
        args::Commands::SyncTemplates => {
            store::sync_templates();
        }
        args::Commands::Render { target } => {
            render::render(target);
        }
        args::Commands::Service { action, name } => {
            service::service_action(&action, &name);
        }
        args::Commands::Autostart { name, toggle } => {
            service::autostart(&name, &toggle);
        }
        args::Commands::RemoveService { name } => {
            service::remove_service(&name);
        }
        args::Commands::Install { package } => {
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
            service::add_service(&name, &cmd, restart_policy.as_deref());
        }
        args::Commands::InstallService(install_args) => {
            service_install::install_service(&install_args);
        }
        args::Commands::ViewLogs { name } => {
            logs::view_logs(&name);
        }
        args::Commands::SaveSettings(settings_args) => {
            settings::save_settings(&settings_args);
        }
        args::Commands::GetMetadata { name } => {
            metadata::get_metadata(&name);
        }
        args::Commands::DetectGpus => {
            let dummy_args = Vec::new();
            gpus::detect_gpus(&dummy_args);
        }
        args::Commands::SetupGpus => {
            let dummy_args = Vec::new();
            gpus::setup_gpu_driver_symlinks(&dummy_args);
        }
        args::Commands::StreamInstall(stream_args) => {
            stream_install::stream_install(&stream_args);
        }
        args::Commands::GetIcon { name } => {
            if let Some(path) = crate::api::utils::get_service_icon_path(&name) {
                println!("{}", path);
            } else {
                println!();
            }
        }
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
