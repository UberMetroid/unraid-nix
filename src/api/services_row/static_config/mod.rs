pub mod part1;
pub mod part2;

pub struct FaIconConfig {
    pub icon: String,
    pub color: &'static str,
    pub bg: &'static str,
    pub border: &'static str,
}

pub struct StaticConfig {
    pub icon: &'static str,
    pub color: &'static str,
    pub bg: &'static str,
    pub border: &'static str,
}

pub fn get_static_config(name_lower: &str) -> StaticConfig {
    if let Some(cfg) = part1::match_static_config(name_lower) {
        return cfg;
    }
    if let Some(cfg) = part2::match_static_config(name_lower) {
        return cfg;
    }

    // Default Generic Server (Grey)
    StaticConfig {
        icon: "fa-server",
        color: "#7f8c8d",
        bg: "rgba(127, 140, 141, 0.08)",
        border: "rgba(127, 140, 141, 0.2)",
    }
}

pub fn get_service_fa_config(name: &str) -> FaIconConfig {
    // Validate service name to prevent path traversal before file I/O.
    if !crate::store::is_valid_service_name(name) {
        let sc = get_static_config(&name.to_lowercase());
        return FaIconConfig {
            icon: sc.icon.to_string(),
            color: sc.color,
            bg: sc.bg,
            border: sc.border,
        };
    }
    let name_lower = name.to_lowercase();
    let meta_file = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    let mut custom_icon = None;
    if let Ok(content) = std::fs::read_to_string(&meta_file) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            custom_icon = val.get("icon")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }

    let sc = get_static_config(&name_lower);
    FaIconConfig {
        icon: custom_icon.unwrap_or_else(|| sc.icon.to_string()),
        color: sc.color,
        bg: sc.bg,
        border: sc.border,
    }
}
