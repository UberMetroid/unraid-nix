use clap::{Parser, Subcommand, Args};

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
    SetupStore {
        persistent_path: String,
    },
    /// Stops services and cleanly unmounts /nix
    #[command(name = "teardown-store")]
    TeardownStore,
    /// Syncs preset templates from the templates repository
    #[command(name = "sync-templates")]
    SyncTemplates,
    /// Renders HTML page templates
    Render {
        #[command(subcommand)]
        target: RenderTargets,
    },
    /// Lifecycle actions (start/stop/restart) for process-compose targets
    Service {
        action: String,
        name: String,
    },
    /// Toggles the autostart setting for a service
    Autostart {
        name: String,
        toggle: String,
    },
    /// Deletes a service definition from the config
    #[command(name = "remove-service")]
    RemoveService {
        name: String,
    },
    /// Installs a package to CLI profile
    Install {
        package: String,
    },
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
    ViewLogs {
        name: String,
    },
    /// Saves Nix plugin settings and manages migration
    #[command(name = "save-settings")]
    SaveSettings(SaveSettingsArgs),
    /// Outputs JSON service metadata
    #[command(name = "get-metadata")]
    GetMetadata {
        name: String,
    },
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
    GetIcon {
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RenderTargets {
    Services,
    Search { query: String },
    Presets,
    Dashboard,
    #[command(name = "dashboard-rows")]
    DashboardRows,
    #[command(name = "dashboard-json")]
    DashboardJson,
    Report { name: String },
}

#[derive(Args, Debug, Clone)]
pub struct SandboxArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub appdata: String,
    #[arg(long)]
    pub media: Option<String>,
    #[arg(long, default_value_t = 99)]
    pub puid: u32,
    #[arg(long, default_value_t = 100)]
    pub pgid: u32,
    #[arg(long)]
    pub gpu: bool,
    #[arg(long)]
    pub gpus: Option<String>,
    #[arg(long)]
    pub cmd: String,
    #[arg(long)]
    pub extra_binds: Option<String>,
    #[arg(long)]
    pub port: Option<String>,
    #[arg(long)]
    pub bind_address: Option<String>,
    #[arg(long)]
    pub network_isolation: bool,
}

#[derive(Args, Debug, Clone)]
pub struct InstallServiceArgs {
    #[arg(long)]
    pub uri: String,
    #[arg(long)]
    pub appdata: String,
    #[arg(long)]
    pub media: Option<String>,
    #[arg(long, default_value_t = 99)]
    pub puid: u32,
    #[arg(long, default_value_t = 100)]
    pub pgid: u32,
    #[arg(long)]
    pub gpu: bool,
    #[arg(long)]
    pub gpus: Option<String>,
    #[arg(long)]
    pub extra_binds: Option<String>,
    #[arg(long)]
    pub port: Option<String>,
    #[arg(long)]
    pub bind_address: Option<String>,
    #[arg(long)]
    pub env_vars: Option<String>,
    #[arg(long)]
    pub compile_locally: bool,
    #[arg(long)]
    pub command_override: Option<String>,
    #[arg(long)]
    pub network_isolation: bool,
}

#[derive(Args, Debug, Clone)]
pub struct SaveSettingsArgs {
    #[arg(long)]
    pub store_path: Option<String>,
    #[arg(long)]
    pub autostart: Option<String>,
    #[arg(long)]
    pub enable_sandbox: Option<String>,
    #[arg(long)]
    pub enable_cli: Option<String>,
    #[arg(long)]
    pub show_in_nav: Option<String>,
    #[arg(long)]
    pub allow_source_builds: Option<String>,
    #[arg(long)]
    pub filter_presets_by_hardware: Option<String>,
    #[arg(long)]
    pub enable_pid_isolation: Option<String>,
    #[arg(long)]
    pub enable_uts_isolation: Option<String>,
    #[arg(long)]
    pub enable_ipc_isolation: Option<String>,
    #[arg(long)]
    pub auto_gc: Option<String>,
    #[arg(long)]
    pub store_quota: Option<String>,
    #[arg(long)]
    pub build_cores: Option<String>,
    #[arg(long)]
    pub build_jobs: Option<String>,
    #[arg(long)]
    pub gc_min_free: Option<String>,
    #[arg(long)]
    pub gc_max_free: Option<String>,
    #[arg(long)]
    pub nix_channel: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct StreamInstallArgs {
    #[arg(long)]
    pub action: String,
    #[arg(long)]
    pub uri: String,
    #[arg(long)]
    pub r#type: Option<String>,
    #[arg(long)]
    pub appdata: Option<String>,
    #[arg(long)]
    pub media: Option<String>,
    #[arg(long)]
    pub puid: Option<String>,
    #[arg(long)]
    pub pgid: Option<String>,
    #[arg(long)]
    pub gpu: Option<String>,
    #[arg(long)]
    pub gpus: Option<String>,
    #[arg(long)]
    pub extra_binds: Option<String>,
    #[arg(long)]
    pub port: Option<String>,
    #[arg(long)]
    pub bind_address: Option<String>,
    #[arg(long)]
    pub env_vars: Option<String>,
    #[arg(long)]
    pub compile_locally: bool,
    #[arg(long)]
    pub command_override: Option<String>,
    #[arg(long)]
    pub network_isolation: Option<String>,
}
