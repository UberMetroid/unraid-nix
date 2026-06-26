use crate::api;

pub fn render(args: &[String]) {
    if args.len() < 3 {
        super::print_usage();
        return;
    }
    match args[2].as_str() {
        "services" => {
            println!("{}", api::render_services_table(29704));
        }
        "search" => {
            let query = if args.len() >= 4 { &args[3] } else { "" };
            println!("{}", api::render_search_results(query));
        }
        "dashboard" => {
            println!("{}", api::render_dashboard_widget(29704));
        }
        _ => super::print_usage(),
    }
}
