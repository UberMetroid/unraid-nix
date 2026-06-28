use std::fs;
use serde::Deserialize;

pub mod category_names;
pub mod category_styling;

use category_names::get_preset_category_name;
use category_styling::get_preset_category_styling;

#[derive(Deserialize)]
#[allow(dead_code)]
struct PresetMeta {
    version: Option<String>,
    license: Option<String>,
    platforms: Option<Vec<String>>,
    maintainers: Option<Vec<String>>,
    programs: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct PresetInfo {
    name: String,
    display_name: String,
    description: String,
    url: String,
    icon: Option<String>,
    command: String,
    #[serde(skip)]
    is_composed: bool,
    composed_parts: Option<Vec<String>>,
    meta: Option<PresetMeta>,
}

fn extract_pkg_name(command: &str, preset_name: &str) -> String {
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

fn should_filter_presets() -> bool {
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

pub fn render_presets_store() -> String {
    let mut presets = Vec::new();

    let filter_enabled = should_filter_presets();
    let detected_gpus = if filter_enabled {
        crate::cli::gpus::get_detected_gpus()
    } else {
        crate::cli::gpus::DetectedGpus {
            has_nvidia: true,
            has_amd: true,
            has_intel: true,
        }
    };

    let scan_dirs = vec![
        ("/usr/local/emhttp/plugins/nix/presets", false),
        ("/usr/local/emhttp/plugins/nix/presets_composed", true),
    ];

    for (dir, is_composed) in scan_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    
                    if filter_enabled {
                        if filename.ends_with("-cuda.json") && !detected_gpus.has_nvidia {
                            continue;
                        }
                        if filename.ends_with("-rocm.json") && !detected_gpus.has_amd {
                            continue;
                        }
                        if filename.ends_with("-vulkan.json") && !detected_gpus.has_intel {
                            continue;
                        }
                    }

                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(mut preset) = serde_json::from_str::<PresetInfo>(&content) {
                            preset.is_composed = is_composed;
                            presets.push(preset);
                        }
                    }
                }
            }
        }
    }

    presets.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));

    let mut html = r##"
    <div class="nix-preset-store-header" style="display: flex; flex-direction: column; gap: 15px; margin-bottom: 20px;">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
            <div>
                <h3 style="margin: 0;">Template Library</h3>
                <p class="nix-subtext" style="margin: 5px 0 0 0;">"##.to_string();
    
    html.push_str(&presets.len().to_string());
    
    html.push_str(r##" verified templates ready for native deployment.</p>
            </div>
            <div style="display: flex; gap: 8px; align-items: center;">
                <!-- Scope Filters -->
                <button type="button" class="nix-scope-btn" onclick="filterPresetScope('composed', this)">Composed</button>
                <button type="button" class="nix-scope-btn" onclick="filterPresetScope('standard', this)">Standard</button>
                
                <!-- Search bar -->
                <div style="position: relative; width: 250px; margin-left: 8px;">
                    <input type="text" id="nix-preset-search" placeholder="Search templates..." onkeyup="filterPresetsStore()" style="width: 100%; padding: 6px 12px 6px 30px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); font-size: 13px; outline: none; transition: border-color 0.15s ease;">
                    <i class="fa fa-search" style="position: absolute; left: 10px; top: 9px; color: var(--nix-text-muted); font-size: 12px;"></i>
                </div>
            </div>
        </div>

        <!-- Category pills (Alphabetically Sorted) -->
        <div class="nix-preset-pills" style="display: flex; gap: 8px; flex-wrap: wrap; padding-bottom: 5px; border-bottom: 1px solid var(--nix-border-primary);">
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('ai', this)">AI</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('automation', this)">Automation</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('backup', this)">Backup</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('cloud', this)">Cloud</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('communication', this)">Communication</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('dashboard', this)">Dashboards</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('database', this)">Databases</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('downloads', this)">Downloaders</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('gaming', this)">Gaming</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('media', this)">Media Players</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('network', this)">Network</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('productivity', this)">Productivity</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('proxy', this)">Proxies</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('security', this)">Security</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('arr', this)">Servarr</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('smarthome', this)">Smart Home</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('social', this)">Social Media</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('sync', this)">Sync</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('vpn', this)">VPN</button>
        </div>
    </div>
    
    <div class="nix-presets-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; margin-bottom: 30px;">
    "##);

    if presets.is_empty() {
        html.push_str(r#"<div style="grid-column: 1 / -1; text-align: center; color: var(--nix-text-muted); padding: 45px 0;">No preset files found on system.</div>"#);
    } else {
        for p in presets {
            let styling = if p.is_composed {
                get_preset_category_styling("composed", p.icon.as_deref().unwrap_or("fa-server"))
            } else {
                get_preset_category_styling(&p.name, p.icon.as_deref().unwrap_or("fa-server"))
            };
            let category_name = if p.is_composed {
                "composed"
            } else {
                get_preset_category_name(&p.name)
            };
            
            let subtitle_html = if let Some(ref parts) = p.composed_parts {
                let tags: Vec<String> = parts.iter().map(|part| {
                    format!(r#"<span style="font-size: 9px; padding: 1px 4px; border-radius: 3px; background: rgba(224, 86, 253, 0.12); border: 1px solid rgba(224, 86, 253, 0.25); color: #e056fd; font-family: monospace;">{}</span>"#, part)
                }).collect();
                format!(r#"<div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 2px;">{}</div>"#, tags.join(""))
            } else {
                format!(r#"<span style="font-size: 10px; color: var(--nix-text-secondary); font-family: monospace;">nixpkgs#{}</span>"#, p.name)
            };
            
            let mut meta_html = String::new();
            if let Some(ref m) = p.meta {
                meta_html.push_str(r#"<div style="display: flex; gap: 5px; margin-top: 4px; flex-wrap: wrap; align-items: center;">"#);
                if let Some(ref v) = m.version {
                    if !v.is_empty() {
                        meta_html.push_str(&format!(
                            r#"<span style="font-size: 8px; color: var(--nix-text-bright); background: var(--nix-bg-tertiary); padding: 1px 4px; border-radius: 3px; border: 1px solid var(--nix-border-primary); display: inline-flex; align-items: center; gap: 2px;" title="Version"><i class="fa fa-tag" style="font-size: 7px;"></i> {}</span>"#,
                            v
                        ));
                    }
                }
                if let Some(ref lic) = m.license {
                    if !lic.is_empty() {
                        meta_html.push_str(&format!(
                            r#"<span style="font-size: 8px; color: var(--nix-text-secondary); background: var(--nix-bg-secondary); padding: 1px 4px; border-radius: 3px; border: 1px solid var(--nix-border-primary); display: inline-flex; align-items: center; gap: 2px;" title="License"><i class="fa fa-gavel" style="font-size: 7px;"></i> {}</span>"#,
                            lic
                        ));
                    }
                }
                if let Some(ref plats) = m.platforms {
                    if !plats.is_empty() {
                        let plat_label = if plats.contains(&"aarch64-linux".to_string()) && plats.contains(&"x86_64-linux".to_string()) {
                            "multi-arch"
                        } else if plats.contains(&"x86_64-linux".to_string()) {
                            "x86_64"
                        } else {
                            "arm64"
                        };
                        meta_html.push_str(&format!(
                            r#"<span style="font-size: 8px; color: var(--nix-text-secondary); background: var(--nix-bg-secondary); padding: 1px 4px; border-radius: 3px; border: 1px solid var(--nix-border-primary); display: inline-flex; align-items: center; gap: 2px;" title="Supported Platforms"><i class="fa fa-laptop" style="font-size: 7px;"></i> {}</span>"#,
                            plat_label
                        ));
                    }
                }
                if let Some(ref progs) = m.programs {
                    if !progs.is_empty() {
                        let progs_str = progs.join(", ");
                        meta_html.push_str(&format!(
                            r#"<div style="font-size: 8px; color: var(--nix-text-muted); margin-top: 4px; width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: inline-flex; align-items: center; gap: 3px;" title="Programs Provided: {}"><i class="fa fa-terminal" style="font-size: 7px; color: var(--nix-text-muted);"></i> {}</div>"#,
                            progs_str, progs_str
                        ));
                    }
                }
                meta_html.push_str("</div>");
            }

            let pkg_search_name = extract_pkg_name(&p.command, &p.name);

             html.push_str(&format!(
                r#"<div class="nix-preset-card" data-name="{}" data-desc="{}" data-category="{}" data-is-composed="{}" style="background: var(--nix-bg-secondary); border: 1px solid var(--nix-border-primary); border-radius: 6px; padding: 16px; display: flex; flex-direction: column; justify-content: space-between; transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease; height: 210px;">
                    <div>
                        <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 12px;">
                            <div style="width: 32px; height: 32px; border-radius: 4px; background: {}; border: 1px solid {}; display: flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0;">
                                <i class="fa {}" style="font-size: 15px;"></i>
                            </div>
                            <div style="display: flex; flex-direction: column; overflow: hidden; width: 100%;">
                                <strong style="font-size: 14px; color: var(--nix-text-primary); text-overflow: ellipsis; white-space: nowrap; overflow: hidden;" title="{}">{}</strong>
                                {}
                                {}
                            </div>
                        </div>
                        <p style="font-size: 12px; color: var(--nix-text-secondary); line-height: 1.5; margin: 0; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; height: 54px;">{}</p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 12px; padding-top: 8px; border-top: 1px solid var(--nix-border-primary);">
                        <div style="display: flex; gap: 10px; align-items: center;">
                            <a href="{}" target="_blank" style="font-size: 11px; color: var(--nix-accent); text-decoration: none; display: inline-flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
                                 <i class="fa fa-globe"></i> Website
                            </a>
                            <a href="https://search.nixos.org/packages?channel=unstable&show={}&query={}" target="_blank" style="font-size: 11px; color: var(--nix-text-muted); text-decoration: none; display: inline-flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();" class="nix-store-link">
                                 <i class="fa fa-book"></i> Nix Store
                            </a>
                        </div>
                        <button type="button" class="nix-btn-install" style="margin: 0; padding: 4px 10px; font-size: 11px; border-radius: 3px;" onclick="showServiceModal('nixpkgs#{}')">
                            <i class="fa fa-plus" style="margin-right: 4px;"></i> Add Service
                        </button>
                    </div>
                </div>"#,
                p.name, p.description.to_lowercase(), category_name, if p.is_composed { "true" } else { "false" }, styling.bg, styling.border, styling.color, styling.icon, p.display_name, p.display_name, subtitle_html, meta_html, p.description, p.url, pkg_search_name, pkg_search_name, p.name
            ));
        }
    }

    html.push_str(r##"</div>
    <script>
    var activeScopes = [];
    var activeCategories = [];

    function filterPresetScope(scope, btn) {
        var idx = activeScopes.indexOf(scope);
        if (idx > -1) {
            activeScopes.splice(idx, 1);
            btn.classList.remove('active');
        } else {
            activeScopes.push(scope);
            btn.classList.add('active');
        }
        applyPresetFilters();
    }

    function filterPresetCategory(cat, btn) {
        var idx = activeCategories.indexOf(cat);
        if (idx > -1) {
            activeCategories.splice(idx, 1);
            btn.classList.remove('active');
        } else {
            activeCategories.push(cat);
            btn.classList.add('active');
        }
        applyPresetFilters();
    }

    function filterPresetsStore() {
        applyPresetFilters();
    }

    function applyPresetFilters() {
        var q = $("#nix-preset-search").val().toLowerCase().trim();
        var cards = document.querySelectorAll('.nix-preset-card');
        cards.forEach(function(card) {
            var name = card.getAttribute('data-name');
            var desc = card.getAttribute('data-desc');
            var category = card.getAttribute('data-category');
            var isComposed = card.getAttribute('data-is-composed') === 'true';
            
            var matchesQuery = (name.indexOf(q) !== -1 || desc.indexOf(q) !== -1);
            
            var matchesScope = false;
            if (activeScopes.length === 0 || activeScopes.length === 2) {
                matchesScope = true;
            } else {
                var selectedScope = activeScopes[0];
                if (selectedScope === 'composed') {
                    matchesScope = isComposed;
                } else if (selectedScope === 'standard') {
                    matchesScope = !isComposed;
                }
            }
            
            var matchesCategory = false;
            if (activeCategories.length === 0) {
                matchesCategory = true;
            } else {
                matchesCategory = activeCategories.indexOf(category) > -1;
            }
            
            if (matchesQuery && matchesScope && matchesCategory) {
                card.style.display = 'flex';
            } else {
                card.style.display = 'none';
            }
        });
    }

    setTimeout(applyPresetFilters, 50);
    </script>
    <style>
    .nix-preset-card:hover {
        transform: translateY(-2px);
        border-color: var(--nix-accent) !important;
        background: var(--nix-bg-tertiary) !important;
        box-shadow: var(--nix-card-shadow);
    }
    #nix-preset-search:focus {
        border-color: var(--nix-accent) !important;
    }
    .nix-preset-pill {
        background: var(--nix-bg-secondary);
        border: 1px solid var(--nix-border-primary);
        border-radius: 12px;
        padding: 4px 12px;
        color: var(--nix-text-secondary);
        font-size: 11px;
        cursor: pointer;
        outline: none;
        margin: 0;
    }
    .nix-preset-pill:hover {
        background: var(--nix-bg-tertiary);
        color: var(--nix-text-primary);
    }
    .nix-preset-pill.active {
        background: rgba(0, 161, 255, 0.1) !important;
        border-color: var(--nix-accent) !important;
        color: var(--nix-accent) !important;
    }
    .nix-store-link:hover {
        color: var(--nix-text-bright) !important;
        text-decoration: underline !important;
    }
    .nix-scope-btn {
        background: var(--nix-bg-secondary);
        border: 1px solid var(--nix-border-primary);
        border-radius: 12px;
        padding: 4px 12px;
        color: var(--nix-text-secondary);
        font-size: 11px;
        cursor: pointer;
        margin: 0;
        outline: none;
    }
    .nix-scope-btn:hover {
        background: var(--nix-bg-tertiary);
        color: var(--nix-text-primary);
    }
    .nix-scope-btn.active {
        background: rgba(0, 161, 255, 0.1) !important;
        border-color: var(--nix-accent) !important;
        color: var(--nix-accent) !important;
    }
    </style>
    "##);

    html
}
