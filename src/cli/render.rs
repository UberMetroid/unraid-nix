use crate::api;
use crate::cli::args::RenderTargets;
use crate::unraid::SUPERVISOR_PORT;

pub fn render(target: RenderTargets) {
    match target {
        RenderTargets::Services => {
            println!("{}", api::render_services_table(SUPERVISOR_PORT));
        }
        RenderTargets::Presets => {
            println!("{}", api::render_presets_store());
        }
        RenderTargets::Search { query } => {
            println!("{}", api::render_search_results(&query));
        }
        RenderTargets::Dashboard => {
            println!("{}", api::render_dashboard_widget(SUPERVISOR_PORT));
        }
        RenderTargets::DashboardRows => {
            println!("{}", api::render_dashboard_rows(SUPERVISOR_PORT));
        }
        RenderTargets::DashboardJson => {
            println!("{}", api::render_dashboard_json(SUPERVISOR_PORT));
        }
        RenderTargets::Report { name } => {
            println!("{}", api::render_verification_report(&name));
        }
    }
}
