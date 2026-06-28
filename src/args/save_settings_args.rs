use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct SaveSettingsArgs {
    #[arg(long)]
    pub store_path: Option<String>,
    #[arg(long)]
    pub autostart: Option<String>,
    #[arg(long)]
    pub enable_sandbox: Option<String>,
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
    pub build_cores: Option<String>,
    #[arg(long)]
    pub build_jobs: Option<String>,
    #[arg(long)]
    pub gc_min_free: Option<String>,
    #[arg(long)]
    pub gc_max_free: Option<String>,
    #[arg(long)]
    pub nix_channel: Option<String>,
    #[arg(long)]
    pub default_appdata_path: Option<String>,
}
