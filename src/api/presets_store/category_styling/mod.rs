pub mod part1;
pub mod part2;

pub struct CategoryStyling {
    pub icon: String,
    pub color: &'static str,
    pub bg: &'static str,
    pub border: &'static str,
}

pub fn get_preset_category_styling(name: &str, default_icon: &str) -> CategoryStyling {
    let name_lower = name.to_lowercase();

    if let Some(styling) = part1::match_styling(&name_lower, default_icon) {
        return styling;
    }
    if let Some(styling) = part2::match_styling(&name_lower, default_icon) {
        return styling;
    }

    // Default Generic Server (Grey)
    CategoryStyling {
        icon: default_icon.to_string(),
        color: "#7f8c8d",
        bg: "rgba(127, 140, 141, 0.08)",
        border: "rgba(127, 140, 141, 0.2)",
    }
}
