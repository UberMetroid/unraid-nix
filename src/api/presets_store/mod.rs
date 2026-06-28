use serde::Deserialize;
use crate::unraid::NIX_CFG_PATH;

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
        let start_byte = pos + "nixpkgs#".len();
        let after = &command[start_byte..];
        let pkg: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
            .collect();
        if !pkg.is_empty() {
            return pkg;
        }
    }
    preset_name.to_string()
}

pub fn should_filter_presets() -> bool {
    if let Ok(content) = std::fs::read_to_string(NIX_CFG_PATH) {
        for line in content.lines() {
            if line.starts_with("FILTER_PRESETS_BY_HARDWARE=") {
                let val = line.trim_start_matches("FILTER_PRESETS_BY_HARDWARE=").trim_matches('"');
                return val == "yes";
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::extract_pkg_name;

    #[test]
    fn test_extract_pkg_name_simple() {
        assert_eq!(extract_pkg_name("nixpkgs#radarr", "radarr"), "radarr");
    }

    #[test]
    fn test_extract_pkg_name_with_version_in_command() {
        assert_eq!(extract_pkg_name("nixpkgs#hello --foo bar", "hello"), "hello");
    }

    #[test]
    fn test_extract_pkg_name_with_dots_and_dashes() {
        assert_eq!(extract_pkg_name("nixpkgs#my.app-name_v2", "x"), "my.app-name_v2");
    }

    #[test]
    fn test_extract_pkg_name_falls_back_when_no_nixpkgs_prefix() {
        assert_eq!(extract_pkg_name("github:owner/repo", "fallback"), "fallback");
    }

    #[test]
    fn test_extract_pkg_name_multibyte_utf8_does_not_panic() {
        let pkg = extract_pkg_name("nixpkgs#中文包名", "fallback");
        assert_eq!(pkg, "中文包名");
    }

    #[test]
    fn test_extract_pkg_name_stops_at_first_non_identifier_char() {
        assert_eq!(extract_pkg_name("nixpkgs#jellyfin-web/jellyfin", "j"), "jellyfin-web");
    }
}
