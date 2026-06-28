use clap::{Parser, Subcommand};

mod install_service_args;
mod sandbox_args;
mod save_settings_args;
mod stream_install_args;

pub use install_service_args::InstallServiceArgs;
pub use sandbox_args::SandboxArgs;
pub use save_settings_args::SaveSettingsArgs;
pub use stream_install_args::StreamInstallArgs;

#[derive(Parser, Debug)]
#[command(name = "nix-helper")]
#[command(about = "Unraid Nix CLI helper", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initializes and bind-mounts /nix
    #[command(name = "setup-store")]
    SetupStore { persistent_path: String },
    /// Stops services and cleanly unmounts /nix
    #[command(name = "teardown-store")]
    TeardownStore,
    /// Syncs preset templates from the templates repository
    #[command(name = "sync-templates")]
    SyncTemplates,
    /// Probes whether Nix's per-derivation build sandbox is functional in
    /// this environment. Reports a JSON object with primitive checks
    /// (user-namespace support, mount-propagation on /nix) and a
    /// 10-second build probe. Use --apply-fallback to write
    /// `sandbox = false` to nix.cfg if the build probe fails.
    #[command(name = "sandbox-check")]
    SandboxCheck {
        /// If set, writes `sandbox = false` to nix.cfg when the build
        /// probe fails. Off by default; the subcommand is reports-only
        /// unless this flag is passed.
        #[arg(long, default_value_t = false)]
        apply_fallback: bool,
    },
    /// Renders HTML page templates
    Render {
        #[command(subcommand)]
        target: RenderTargets,
    },
    /// Lifecycle actions (start/stop/restart) for process-compose targets
    Service { action: String, name: String },
    /// Toggles the autostart setting for a service
    Autostart { name: String, toggle: String },
    /// Deletes a service definition from the config
    #[command(name = "remove-service")]
    RemoveService { name: String },
    /// Installs a package to CLI profile
    Install { package: String },
    /// Helper command to print bubblewrap script
    Sandbox(SandboxArgs),
    /// Helper command to print preset bubblewrap script
    Preset {
        name: String,
        appdata: String,
        media: String,
        puid: u32,
        pgid: u32,
        gpu: String,
        extra_binds: Option<String>,
        port: Option<String>,
        bind_address: Option<String>,
    },
    /// Adds a service to process-compose configuration
    #[command(name = "add-service")]
    AddService {
        name: String,
        cmd: String,
        restart_policy: Option<String>,
    },
    /// Installs a service, creates folders/metadata, and adds it
    #[command(name = "install-service")]
    InstallService(InstallServiceArgs),
    /// Outputs formatted service console logs
    #[command(name = "view-logs")]
    ViewLogs { name: String },
    /// Saves Nix plugin settings and manages migration
    #[command(name = "save-settings")]
    SaveSettings(SaveSettingsArgs),
    /// Outputs JSON service metadata
    #[command(name = "get-metadata")]
    GetMetadata { name: String },
    /// Outputs JSON list of detected host GPUs
    #[command(name = "detect-gpus")]
    DetectGpus,
    /// Configures NVIDIA/CUDA symlinks on host
    #[command(name = "setup-gpus")]
    SetupGpus,
    /// Streams real-time installation output and tails logs
    #[command(name = "stream-install")]
    StreamInstall(StreamInstallArgs),
    /// Outputs the absolute path of a service logo in the Nix store
    #[command(name = "get-icon")]
    GetIcon { name: String },
    /// Checks the status of the Nix daemon
    #[command(name = "daemon-status")]
    DaemonStatus,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RenderTargets {
    Services,
    Search {
        query: String,
    },
    Presets,
    Dashboard,
    #[command(name = "dashboard-rows")]
    DashboardRows,
    #[command(name = "dashboard-json")]
    DashboardJson,
    Report {
        name: String,
    },
}
