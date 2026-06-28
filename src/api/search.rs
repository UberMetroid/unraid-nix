use crate::search::search_packages;

/// Renders search results from the Nixpkgs registry into an HTML table.
pub fn render_search_results(query: &str) -> String {
    let results = match search_packages(query) {
        Ok(r) => r,
        Err(e) => return format!(r#"<div class="alert alert-danger"><i class="fa fa-times"></i> Search failed: {}</div>"#, e),
    };

    let mut html = r#"<table class="nix-table">
            <thead>
                <tr>
                    <th style="width: 250px;">Package Name</th>
                    <th>Description</th>
                    <th style="width: 220px;">Details</th>
                    <th style="width: 130px; text-align: center;">Action</th>
                </tr>
            </thead>
            <tbody>"#.to_string();

    if results.is_empty() {
        html.push_str(r#"<tr><td colspan="4" class="text-center">No packages found matching your query.</td></tr>"#);
    } else {
        for r in results {
            let action_buttons = format!(
                r#"<button type="button" class="nix-btn-install" style="width: 100px; margin: 0; padding: 4px 8px; font-size: 11px;" onclick="showServiceModal('{}')">Add Service</button>"#,
                r.package_name
            );

            let mut meta_links = Vec::new();
            if let Some(ref lic) = r.license {
                if !lic.trim().is_empty() {
                    meta_links.push(format!(r#"<span><i class="fa fa-certificate" style="margin-right: 3px;"></i>{}</span>"#, lic));
                }
            }
            if let Some(ref hp) = r.homepage {
                if !hp.trim().is_empty() {
                    meta_links.push(format!(r#"<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;"><i class="fa fa-globe" style="margin-right: 3px;"></i>Homepage</a>"#, hp));
                }
            }
            if let Some(ref pos) = r.position {
                if !pos.trim().is_empty() {
                    meta_links.push(format!(r#"<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;"><i class="fa fa-code" style="margin-right: 3px;"></i>Source</a>"#, pos));
                }
            }

            let meta_html = if meta_links.is_empty() {
                "".to_string()
            } else {
                format!(
                    r#"<div style="margin-top: 6px; font-size: 11px; display: flex; gap: 12px; flex-wrap: wrap; align-items: center; color: var(--nix-text-secondary);">{}</div>"#,
                    meta_links.join(r#"<span style="color: var(--nix-text-muted);">|</span>"#)
                )
            };

            let description_cell = format!("<div>{}</div>{}", r.description, meta_html);

            let short_name = r.package_name.replace("nixpkgs#", "");
            let link_url = crate::api::package::get_package_link_url(&r.package_name)
                .unwrap_or_else(|| format!("https://search.nixos.org/packages?channel=unstable&show={}&query={}", short_name, short_name));
            let package_link = format!(
                r#"<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;"><code>{}</code> <i class="fa fa-external-link" style="font-size: 10px; margin-left: 2px;"></i></a>"#,
                link_url, short_name
            );

            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                package_link, r.version, description_cell, action_buttons
            ));
        }
    }

    html.push_str("</tbody></table></div>");
    html
}
