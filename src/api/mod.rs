pub mod dashboard;
pub mod package;
pub mod presets_store;
pub mod report;
pub mod search;
pub mod services;
pub mod services_row;
pub mod utils;

pub use dashboard::dashboard_diff;
pub use dashboard::render_dashboard_json;
pub use dashboard::render_dashboard_rows;
pub use dashboard::render_dashboard_widget;
pub use presets_store::render_presets_store;
pub use report::render_verification_report;
pub use search::render_search_results;
pub use services::render_services_table;

#[cfg(test)]
mod tests {
    use super::package::get_package_link_url;

    #[test]
    fn test_get_package_link_url() {
        assert_eq!(
            get_package_link_url("nixpkgs#sonarr"),
            Some(
                "https://search.nixos.org/packages?channel=unstable&show=sonarr&query=sonarr"
                    .to_string()
            )
        );
        assert_eq!(
            get_package_link_url("github:numtide/blueprint#my-service"),
            Some("https://github.com/numtide/blueprint".to_string())
        );
        assert_eq!(
            get_package_link_url("github:numtide/blueprint"),
            Some("https://github.com/numtide/blueprint".to_string())
        );
        assert_eq!(get_package_link_url("/path/to/local/flake"), None);
    }
}
