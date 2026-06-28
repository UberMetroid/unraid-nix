use clap::Args;

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