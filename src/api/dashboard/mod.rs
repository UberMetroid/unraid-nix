pub mod diff;
pub mod rows;
pub mod widget;

pub use diff::dashboard_diff;
pub use rows::render_dashboard_json;
pub use rows::render_dashboard_rows;
pub use widget::render_dashboard_widget;
