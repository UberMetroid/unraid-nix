use crate::unraid::{parse_ini_file, NIX_CFG_PATH};

pub(crate) const BYTES_PER_GB: u64 = 1 << 30;

pub fn generate_nix_conf_content(
    allow_source: bool,
    build_cores: &str,
    build_jobs: &str,
    gc_min_free_gb: u64,
    gc_max_free_gb: u64,
) -> Result<String, String> {
    let (jobs_val, cores_val) = if allow_source {
        let j = if build_jobs == "0" {
            let total = std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4);
            let half = total.saturating_add(1) / 2;
            std::cmp::max(1, half).to_string()
        } else {
            build_jobs.to_string()
        };
        (j, build_cores.to_string())
    } else {
        ("0".to_string(), "0".to_string())
    };

    let min_free_bytes = gc_min_free_gb.checked_mul(BYTES_PER_GB)
        .ok_or_else(|| "min_free_gb overflow".to_string())?;
    let max_free_bytes = gc_max_free_gb.checked_mul(BYTES_PER_GB)
        .ok_or_else(|| "max_free_gb overflow".to_string())?;

    // `NIX_BUILD_SANDBOX` opt-out. Default: enabled (true). Admin can set
    // this to "no" in nix.cfg to disable Nix's per-derivation build
    // sandbox. The runtime `nix-helper sandbox-check` subcommand reports
    // whether the sandbox actually works in this environment.
    let sandbox_enabled = parse_ini_file(NIX_CFG_PATH)
        .get("NIX_BUILD_SANDBOX")
        .map(|v| !matches!(v.to_lowercase().as_str(), "no" | "false" | "0" | "off"))
        .unwrap_or(true);
    let sandbox_setting = if sandbox_enabled { "true" } else { "false" };

    Ok(format!(
        "sandbox = {sandbox_setting}\nexperimental-features = nix-command flakes\nmax-jobs = {jobs_val}\ncores = {cores_val}\nmin-free = {min_free_bytes}\nmax-free = {max_free_bytes}\n",
        sandbox_setting = sandbox_setting,
        jobs_val = jobs_val,
        cores_val = cores_val,
        min_free_bytes = min_free_bytes,
        max_free_bytes = max_free_bytes,
    ))
}