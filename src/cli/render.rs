use crate::api;
use crate::cli::args::RenderTargets;

pub fn render(target: RenderTargets) {
    match target {
        RenderTargets::Services => {
            println!("{}", api::render_services_table(29704));
        }
        RenderTargets::Presets => {
            println!("{}", api::render_presets_store());
        }
        RenderTargets::Search { query } => {
            println!("{}", api::render_search_results(&query));
        }
        RenderTargets::Dashboard => {
            println!("{}", api::render_dashboard_widget(29704));
        }
        RenderTargets::DashboardRows => {
            println!("{}", api::render_dashboard_rows(29704));
        }
        RenderTargets::DashboardJson => {
            println!("{}", api::render_dashboard_json(29704));
        }
        RenderTargets::Report { name } => {
            println!("{}", api::render_verification_report(&name));
        }
    }
}
