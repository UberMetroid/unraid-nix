use serde::Deserialize;

pub mod category_names;
pub mod category_styling;
pub mod renderer;

pub use renderer::render_presets_store;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PresetMeta {
    pub version: Option<String>,
    pub license: Option<String>,
    pub platforms: Option<Vec<String>>,
    pub maintainers: Option<Vec<String>>,
    pub programs: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct PresetInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub url: String,
    pub icon: Option<String>,
    pub command: String,
    #[serde(skip)]
    pub is_composed: bool,
    pub composed_parts: Option<Vec<String>>,
    pub meta: Option<PresetMeta>,
}

pub fn extract_pkg_name(command: &str, preset_name: &str) -> String {
    if let Some(pos) = command.find("nixpkgs#") {
        let start = pos + 8;
        let mut end = start;
        let chars: Vec<char> = command.chars().collect();
        while end < chars.len() {
            let c = chars[end];
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                end += 1;
            } else {
                break;
            }
        }
        if end > start {
            return command[start..end].to_string();
        }
    }
    preset_name.to_string()
}

pub fn should_filter_presets() -> bool {
    if let Ok(content) = std::fs::read_to_string("/boot/config/plugins/nix/nix.cfg") {
        for line in content.lines() {
            if line.starts_with("FILTER_PRESETS_BY_HARDWARE=") {
                let val = line.trim_start_matches("FILTER_PRESETS_BY_HARDWARE=").trim_matches('"');
                return val == "yes";
            }
        }
    }
    true // Defaults to true
}
