use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct PresetInfo {
    name: String,
    display_name: String,
    description: String,
    url: String,
    icon: Option<String>,
}

pub fn render_presets_store() -> String {
    let presets_dir = "/usr/local/emhttp/plugins/nix/presets";
    let mut presets = Vec::new();

    if let Ok(entries) = fs::read_dir(presets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
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

        <!-- Category pills (Alphabetically Sorted, with AI first) -->
        <div class="nix-preset-pills" style="display: flex; gap: 8px; flex-wrap: wrap; padding-bottom: 5px; border-bottom: 1px solid rgba(255,255,255,0.05);">
            <button type="button" class="nix-preset-pill active" onclick="filterPresetCategory('ai', this)">AI</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('automation', this)">ARR!</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('database', this)">Databases</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('downloads', this)">Downloads</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('media', this)">Media & Audio</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('network', this)">Network & VPN</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('security', this)">Security & Locks</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('smarthome', this)">Smart Home</button>
            <button type="button" class="nix-preset-pill" onclick="filterPresetCategory('storage', this)">Sync & Backups</button>
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
            var matchesCategory = (category === activeCategory);
            
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

struct CategoryStyling {
    icon: String,
    color: &'static str,
    bg: &'static str,
    border: &'static str,
}

fn get_preset_category_name(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();
    if name_lower.contains("ollama") || name_lower.contains("open-webui") || name_lower.contains("localai") ||
       name_lower.contains("anythingllm") || name_lower.contains("librechat") || name_lower.contains("flowise") ||
       name_lower.contains("stable-diffusion") || name_lower.contains("comfyui") || name_lower.contains("text-generation-webui") ||
       name_lower.contains("invokeai") || name_lower.contains("fooocus") || name_lower.contains("dify") || name_lower.contains("lobe-chat") {
        "ai"
    } else if name_lower.contains("jellyfin") || name_lower.contains("plex") || name_lower.contains("emby") ||
       name_lower.contains("navidrome") || name_lower.contains("airsonic") || name_lower.contains("subsonic") || name_lower.contains("lidarr") {
        "media"
    } else if name_lower.contains("sonarr") || name_lower.contains("sickrage") || name_lower.contains("sickchill") ||
       name_lower.contains("radarr") || name_lower.contains("couchpotato") || name_lower.contains("readarr") ||
       name_lower.contains("calibre") || name_lower.contains("audiobookshelf") || name_lower.contains("bazarr") ||
       name_lower.contains("prowlarr") || name_lower.contains("jackett") {
        "automation"
    } else if name_lower.contains("transmission") || name_lower.contains("sabnzbd") || name_lower.contains("nzbget") || 
       name_lower.contains("qbittorrent") || name_lower.contains("deluge") || name_lower.contains("rtorrent") || name_lower.contains("aria2") {
        "downloads"
    } else if name_lower.contains("pihole") || name_lower.contains("pi-hole") || name_lower.contains("adguard") ||
       name_lower.contains("nginx") || name_lower.contains("traefik") || name_lower.contains("caddy") || name_lower.contains("npm") ||
       name_lower.contains("tailscale") || name_lower.contains("wireguard") || name_lower.contains("vpn") {
        "network"
    } else if name_lower.contains("home-assistant") || name_lower.contains("homeassistant") || name_lower.contains("hass") ||
       name_lower.contains("node-red") || name_lower.contains("nodered") || name_lower.contains("zigbee") ||
       name_lower.contains("mqtt") || name_lower.contains("esphome") {
        "smarthome"
    } else if name_lower.contains("vaultwarden") || name_lower.contains("bitwarden") || name_lower.contains("keepass") {
        "security"
    } else if name_lower.contains("syncthing") || name_lower.contains("nextcloud") || name_lower.contains("owncloud") ||
       name_lower.contains("seafile") || name_lower.contains("rclone") || name_lower.contains("duplicati") ||
       name_lower.contains("kopia") || name_lower.contains("backups") {
        "storage"
    } else if name_lower.contains("influx") || name_lower.contains("prometheus") || name_lower.contains("grafana") ||
       name_lower.contains("kuma") || name_lower.contains("netdata") || name_lower.contains("postgres") ||
       name_lower.contains("mysql") || name_lower.contains("mariadb") || name_lower.contains("redis") ||
       name_lower.contains("mongo") {
        "database"
    } else {
        "default"
    }
}

fn get_preset_category_styling(name: &str, default_icon: &str) -> CategoryStyling {
    let name_lower = name.to_lowercase();
    
    // AI & LLMs (Indigo/Cyberpunk Purple-Blue)
    if name_lower.contains("ollama") || name_lower.contains("open-webui") || name_lower.contains("localai") ||
       name_lower.contains("anythingllm") || name_lower.contains("librechat") || name_lower.contains("flowise") ||
       name_lower.contains("stable-diffusion") || name_lower.contains("comfyui") || name_lower.contains("text-generation-webui") ||
       name_lower.contains("invokeai") || name_lower.contains("fooocus") || name_lower.contains("dify") || name_lower.contains("lobe-chat") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#a29bfe",
            bg: "rgba(162, 155, 254, 0.08)",
            border: "rgba(162, 155, 254, 0.2)",
        };
    }

    // Media & Audio (Cyan/Blue)
    if name_lower.contains("jellyfin") || name_lower.contains("plex") || name_lower.contains("emby") ||
       name_lower.contains("navidrome") || name_lower.contains("airsonic") || name_lower.contains("subsonic") || name_lower.contains("lidarr") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#00a1ff",
            bg: "rgba(0, 161, 255, 0.08)",
            border: "rgba(0, 161, 255, 0.2)",
        };
    }
    
    // Servarr / Automation (Orange)
    if name_lower.contains("sonarr") || name_lower.contains("sickrage") || name_lower.contains("sickchill") ||
       name_lower.contains("radarr") || name_lower.contains("couchpotato") || name_lower.contains("readarr") ||
       name_lower.contains("calibre") || name_lower.contains("audiobookshelf") || name_lower.contains("bazarr") ||
       name_lower.contains("prowlarr") || name_lower.contains("jackett") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#e67e22",
            bg: "rgba(230, 126, 34, 0.08)",
            border: "rgba(230, 126, 34, 0.2)",
        };
    }

    // Download Clients (Green)
    if name_lower.contains("transmission") || name_lower.contains("sabnzbd") || name_lower.contains("nzbget") || 
       name_lower.contains("qbittorrent") || name_lower.contains("deluge") || name_lower.contains("rtorrent") || name_lower.contains("aria2") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#2ecc71",
            bg: "rgba(46, 204, 113, 0.08)",
            border: "rgba(46, 204, 113, 0.2)",
        };
    }

    // Network / Security / VPN (Purple)
    if name_lower.contains("pihole") || name_lower.contains("pi-hole") || name_lower.contains("adguard") ||
       name_lower.contains("nginx") || name_lower.contains("traefik") || name_lower.contains("caddy") || name_lower.contains("npm") ||
       name_lower.contains("tailscale") || name_lower.contains("wireguard") || name_lower.contains("vpn") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#9b59b6",
            bg: "rgba(155, 89, 182, 0.08)",
            border: "rgba(155, 89, 182, 0.2)",
        };
    }

    // Smart Home & IoT (Yellow)
    if name_lower.contains("home-assistant") || name_lower.contains("homeassistant") || name_lower.contains("hass") ||
       name_lower.contains("node-red") || name_lower.contains("nodered") || name_lower.contains("zigbee") ||
       name_lower.contains("mqtt") || name_lower.contains("esphome") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#f1c40f",
            bg: "rgba(241, 196, 15, 0.08)",
            border: "rgba(241, 196, 15, 0.2)",
        };
    }

    // Vaults & Passwords (Red)
    if name_lower.contains("vaultwarden") || name_lower.contains("bitwarden") || name_lower.contains("keepass") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#e74c3c",
            bg: "rgba(231, 76, 60, 0.08)",
            border: "rgba(231, 76, 60, 0.2)",
        };
    }

    // Sync & Backups (Teal)
    if name_lower.contains("syncthing") || name_lower.contains("nextcloud") || name_lower.contains("owncloud") ||
       name_lower.contains("seafile") || name_lower.contains("rclone") || name_lower.contains("duplicati") ||
       name_lower.contains("kopia") || name_lower.contains("backups") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#1abc9c",
            bg: "rgba(26, 188, 156, 0.08)",
            border: "rgba(26, 188, 156, 0.2)",
        };
    }

    // Databases & Monitoring (Grey-Blue)
    if name_lower.contains("influx") || name_lower.contains("prometheus") || name_lower.contains("grafana") ||
       name_lower.contains("kuma") || name_lower.contains("netdata") || name_lower.contains("postgres") ||
       name_lower.contains("mysql") || name_lower.contains("mariadb") || name_lower.contains("redis") ||
       name_lower.contains("mongo") {
        return CategoryStyling {
            icon: default_icon.to_string(),
            color: "#6c7a89",
            bg: "rgba(108, 122, 137, 0.08)",
            border: "rgba(108, 122, 137, 0.2)",
        };
    }

    // Default Generic Server (Grey)
    CategoryStyling {
        icon: default_icon.to_string(),
        color: "#7f8c8d",
        bg: "rgba(127, 140, 141, 0.08)",
        border: "rgba(127, 140, 141, 0.2)",
    }
}
