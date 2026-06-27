use std::fs;
use serde::Deserialize;

pub mod category_names;
pub mod category_styling;

use category_names::get_preset_category_name;
use category_styling::get_preset_category_styling;

#[derive(Deserialize)]
struct PresetInfo {
    name: String,
    display_name: String,
    description: String,
    url: String,
    icon: Option<String>,
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
    let presets_dir = "/usr/local/emhttp/plugins/nix/presets";
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

    if let Ok(entries) = fs::read_dir(presets_dir) {
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
                    if let Ok(preset) = serde_json::from_str::<PresetInfo>(&content) {
                        presets.push(preset);
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
                <h3 style="margin: 0;">Preset Service Store</h3>
                <p class="nix-subtext" style="margin: 5px 0 0 0;">Browse and configure over 200+ pre-configured self-hosted templates.</p>
            </div>
            <div style="display: flex; gap: 10px; align-items: center;">
                <!-- Search bar -->
                <div style="position: relative; width: 250px;">
                    <input type="text" id="nix-preset-search" placeholder="Search templates..." onkeyup="filterPresetsStore()" style="width: 100%; padding: 6px 12px 6px 30px; border-radius: 4px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: #fff; font-size: 13px; outline: none; transition: border-color 0.15s ease;">
                    <i class="fa fa-search" style="position: absolute; left: 10px; top: 9px; color: #666; font-size: 12px;"></i>
                </div>
            </div>
        </div>

        <!-- Category pills (Alphabetically Sorted) -->
        <div class="nix-preset-pills" style="display: flex; gap: 8px; flex-wrap: wrap; padding-bottom: 5px; border-bottom: 1px solid rgba(255,255,255,0.05);">
            <button type="button" class="nix-preset-pill active" onclick="filterPresetCategory('ai', this)">AI</button>
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
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('security', this)">Security</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('arr', this)">Servarr</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('smarthome', this)">Smart Home</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('social', this)">Social Media</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('sync', this)">Sync</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('vpn', this)">VPN</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('all', this)">All</button>
        </div>
    </div>
    
    <div class="nix-presets-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; margin-bottom: 30px;">
    "##.to_string();

    if presets.is_empty() {
        html.push_str(r#"<div style="grid-column: 1 / -1; text-align: center; color: #888; padding: 45px 0;">No preset files found on system.</div>"#);
    } else {
        for p in presets {
            let styling = get_preset_category_styling(&p.name, p.icon.as_deref().unwrap_or("fa-server"));
            let category_name = get_preset_category_name(&p.name);
            
            html.push_str(&format!(
                r#"<div class="nix-preset-card" data-name="{}" data-desc="{}" data-category="{}" style="background: rgba(255, 255, 255, 0.02); border: 1px solid rgba(255, 255, 255, 0.05); border-radius: 6px; padding: 16px; display: flex; flex-direction: column; justify-content: space-between; transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease; height: 180px;">
                    <div>
                        <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 12px;">
                            <div style="width: 32px; height: 32px; border-radius: 4px; background: {}; border: 1px solid {}; display: flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0;">
                                <i class="fa {}" style="font-size: 15px;"></i>
                            </div>
                            <div style="display: flex; flex-direction: column; overflow: hidden;">
                                <strong style="font-size: 14px; color: #fff; text-overflow: ellipsis; white-space: nowrap; overflow: hidden;" title="{}">{}</strong>
                                <span style="font-size: 10px; color: #a0a0a5; font-family: monospace;">nixpkgs#{}</span>
                            </div>
                        </div>
                        <p style="font-size: 12px; color: #a0a0a5; line-height: 1.5; margin: 0; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; height: 54px;">{}</p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 12px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.03);">
                        <a href="{}" target="_blank" style="font-size: 11px; color: #00a1ff; text-decoration: none; display: inline-flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
                            <i class="fa fa-globe"></i> Website
                        </a>
                        <button type="button" class="nix-btn-install" style="margin: 0; padding: 4px 10px; font-size: 11px; border-radius: 3px;" onclick="showServiceModal('nixpkgs#{}')">
                            <i class="fa fa-plus" style="margin-right: 4px;"></i> Add Service
                        </button>
                    </div>
                </div>"#,
                p.name, p.description.to_lowercase(), category_name, styling.bg, styling.border, styling.color, styling.icon, p.display_name, p.display_name, p.name, p.description, p.url, p.name
            ));
        }
    }

    html.push_str(r##"</div>
    <script>
    var activeCategory = 'ai';

    function filterPresetCategory(cat, btn) {
        activeCategory = cat;
        document.querySelectorAll('.nix-preset-pill').forEach(function(pill) {
            pill.classList.remove('active');
        });
        btn.classList.add('active');
        applyPresetFilters();
    }

    function filterPresetsStore() {
        var q = $("#nix-preset-search").val().trim();
        if (q.length > 0 && activeCategory !== 'all') {
            activeCategory = 'all';
            document.querySelectorAll('.nix-preset-pill').forEach(function(pill) {
                pill.classList.remove('active');
                if (pill.getAttribute('onclick') && pill.getAttribute('onclick').indexOf("'all'") !== -1) {
                    pill.classList.add('active');
                }
            });
        }
        applyPresetFilters();
    }

    function applyPresetFilters() {
        var q = $("#nix-preset-search").val().toLowerCase().trim();
        var cards = document.querySelectorAll('.nix-preset-card');
        cards.forEach(function(card) {
            var name = card.getAttribute('data-name');
            var desc = card.getAttribute('data-desc');
            var category = card.getAttribute('data-category');
            
            var matchesQuery = (name.indexOf(q) !== -1 || desc.indexOf(q) !== -1);
            var matchesCategory = (activeCategory === 'all' || category === activeCategory);
            
            if (matchesQuery && matchesCategory) {
                card.style.display = 'flex';
            } else {
                card.style.display = 'none';
            }
        });
    }

    // Apply default filters immediately on script execution
    setTimeout(applyPresetFilters, 50);
    </script>
    <style>
    .nix-preset-card:hover {
        transform: translateY(-2px);
        border-color: rgba(0, 161, 255, 0.25) !important;
        background: rgba(255, 255, 255, 0.035) !important;
        box-shadow: 0 4px 12px rgba(0,0,0,0.2);
    }
    #nix-preset-search:focus {
        border-color: rgba(0, 161, 255, 0.5) !important;
    }
    .nix-preset-pill {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        padding: 4px 12px;
        color: #a0a0a5;
        font-size: 11px;
        cursor: pointer;
        outline: none;
        transition: all 0.15s ease;
        margin: 0;
    }
    .nix-preset-pill:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
    }
    .nix-preset-pill.active {
        background: rgba(0, 161, 255, 0.1) !important;
        border-color: rgba(0, 161, 255, 0.3) !important;
        color: #00a1ff !important;
        font-weight: 500;
    }
    </style>
    "##);

    html
}
