use clap::Args;

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
