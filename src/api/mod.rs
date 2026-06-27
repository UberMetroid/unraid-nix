pub mod utils;
pub mod services;
pub mod search;
pub mod dashboard;
pub mod package;
pub mod report;
pub mod services_row;

pub use services::render_services_table;
pub use search::render_search_results;
pub use dashboard::render_dashboard_widget;
pub use dashboard::render_dashboard_rows;
pub use report::render_verification_report;

#[cfg(test)]
mod tests {
    use super::package::get_package_link_url;

    #[test]
    fn test_get_package_link_url() {
        assert_eq!(
            get_package_link_url("nixpkgs#sonarr"),
            Some("https://search.nixos.org/packages?channel=unstable&show=sonarr&query=sonarr".to_string())
        );
        assert_eq!(
            get_package_link_url("github:numtide/blueprint#my-service"),
            Some("https://github.com/numtide/blueprint".to_string())
        );
        assert_eq!(
            get_package_link_url("github:numtide/blueprint"),
            Some("https://github.com/numtide/blueprint".to_string())
        );
        assert_eq!(
            get_package_link_url("/path/to/local/flake"),
            None
        );
    }
}
