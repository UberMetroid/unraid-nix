use std::process::exit;

pub fn sandbox_cmd(args: &crate::args::SandboxArgs) {
    let config = crate::sandbox::SandboxConfig {
        name: args.name.clone(),
        appdata_path: args.appdata.clone(),
        media_path: args.media.clone(),
        puid: args.puid,
        pgid: args.pgid,
        enable_gpu: args.gpu,
        gpus: args.gpus.clone(),
        inner_command: args.cmd.clone(),
        extra_binds: args.extra_binds.as_ref()
            .and_then(|s| crate::sandbox::parse_binds_string(s).ok())
            .unwrap_or_default(),
        port: args.port.clone(),
        bind_address: args.bind_address.clone(),
        host_init_commands: Vec::new(),
        enable_network_isolation: args.network_isolation,
    };
    match crate::sandbox::build_bwrap_command(&config) {
        Ok(cmd) => println!("{cmd}"),
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Failed to build bubblewrap sandbox command for '{}': {e}", config.name));
            exit(1);
        }
    }
}