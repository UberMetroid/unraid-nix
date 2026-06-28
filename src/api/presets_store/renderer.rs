use std::fs;

use rayon::prelude::*;

use super::category_names::get_preset_category_name;
use super::category_styling::get_preset_category_styling;
use super::{PresetInfo, extract_pkg_name, should_filter_presets};
use crate::api::utils::{html_escape, js_escape};

pub fn render_presets_store() -> String {
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

    let mut presets = collect_presets(&scan_dirs, filter_enabled, &detected_gpus);

    presets.sort_by_key(|a| a.display_name.to_lowercase());

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
                    <input type="text" id="nix-preset-search" placeholder="Search templates..." onkeyup="filterPresetsStore()" style="width: 100%; padding: 6px 12px 30px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); font-size: 13px; outline: none; transition: border-color 0.15s ease;">
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
                    format!(r#"<span style="font-size: 9px; padding: 1px 4px; border-radius: 3px; background: rgba(224, 86, 253, 0.12); border: 1px solid rgba(224, 86, 253, 0.25); color: #e056fd; font-family: monospace;">{}</span>"#, html_escape(part))
                }).collect();
                format!(r#"<div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 2px;">{}</div>"#, tags.join(""))
            } else {
                format!(r#"<span style="font-size: 10px; color: var(--nix-text-secondary); font-family: monospace;">nixpkgs#{}</span>"#, html_escape(&p.name))
            };

            let mut meta_html = String::new();
            if let Some(ref m) = p.meta {
                meta_html.push_str(r#"<div style="display: flex; gap: 5px; margin-top: 4px; flex-wrap: wrap; align-items: center;">"#);
                if let Some(ref v) = m.version {
                    if !v.is_empty() {
                        meta_html.push_str(&format!(
                            r#"<span style="font-size: 8px; color: var(--nix-text-bright); background: var(--nix-bg-tertiary); padding: 1px 4px; border-radius: 3px; border: 1px solid var(--nix-border-primary); display: inline-flex; align-items: center; gap: 2px;" title="Version"><i class="fa fa-tag" style="font-size: 7px;"></i> {}</span>"#,
                            html_escape(v)
                        ));
                    }
                }
                if let Some(ref lic) = m.license {
                    if !lic.is_empty() {
                        meta_html.push_str(&format!(
                            r#"<span style="font-size: 8px; color: var(--nix-text-secondary); background: var(--nix-bg-secondary); padding: 1px 4px; border-radius: 3px; border: 1px solid var(--nix-border-primary); display: inline-flex; align-items: center; gap: 2px;" title="License"><i class="fa fa-gavel" style="font-size: 7px;"></i> {}</span>"#,
                            html_escape(lic)
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
                            html_escape(&progs_str), html_escape(&progs_str)
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
                html_escape(&p.name), html_escape(&p.description.to_lowercase()), html_escape(category_name), if p.is_composed { "true" } else { "false" }, styling.bg, styling.border, styling.color, html_escape(&styling.icon), html_escape(&p.display_name), html_escape(&p.display_name), subtitle_html, meta_html, html_escape(&p.description), html_escape(&p.url), html_escape(&pkg_search_name), html_escape(&pkg_search_name), html_escape(&js_escape(&p.name))
            ));
        }
    }

    html.push_str(r##"</div>"##);
    html
}

/// Read every `*.json` preset file under the given directories in parallel and
/// deserialize each one into a [`PresetInfo`].
///
/// File I/O and `serde_json::from_str` are read-only and thread-safe, so we
/// fan the work out across rayon's thread pool with `par_iter`. Results are
/// collected into a `Vec<PresetInfo>` and the caller is still expected to sort
/// the result (e.g. by `display_name`) before rendering.
fn collect_presets(
    scan_dirs: &[(&str, bool)],
    filter_enabled: bool,
    detected_gpus: &crate::cli::gpus::DetectedGpus,
) -> Vec<PresetInfo> {
    scan_dirs
        .par_iter()
        .flat_map(|(dir, is_composed)| {
            let entries = match fs::read_dir(dir) {
                Ok(e) => e.flatten().collect::<Vec<_>>(),
                Err(_) => return Vec::new(),
            };

            entries
                .par_iter()
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("json") {
                        return None;
                    }

                    let filename = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");

                    if filter_enabled {
                        if filename.ends_with("-cuda.json") && !detected_gpus.has_nvidia {
                            return None;
                        }
                        if filename.ends_with("-rocm.json") && !detected_gpus.has_amd {
                            return None;
                        }
                        if filename.ends_with("-vulkan.json") && !detected_gpus.has_intel {
                            return None;
                        }
                    }

                    let content = fs::read_to_string(&path).ok()?;
                    let mut preset = serde_json::from_str::<PresetInfo>(&content).ok()?;
                    preset.is_composed = *is_composed;
                    Some(preset)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Count `.json` preset files in a directory (non-recursive). Used by tests
    /// to verify that [`collect_presets`] returns the expected number of presets
    /// for a given directory layout.
    fn count_json_files(dir: &Path) -> usize {
        fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
                    .count()
            })
            .unwrap_or(0)
    }

    fn repo_presets_root() -> PathBuf {
        // CARGO_MANIFEST_DIR points at the crate root, which is also where
        // the checked-in `presets/` and `presets_composed/` directories live.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn all_gpus() -> crate::cli::gpus::DetectedGpus {
        crate::cli::gpus::DetectedGpus {
            has_nvidia: true,
            has_amd: true,
            has_intel: true,
        }
    }

    fn no_gpus() -> crate::cli::gpus::DetectedGpus {
        crate::cli::gpus::DetectedGpus {
            has_nvidia: false,
            has_amd: false,
            has_intel: false,
        }
    }

    #[test]
    fn collect_presets_returns_one_entry_per_json_file_unfiltered() {
        let root = repo_presets_root();
        let presets_dir = root.join("presets");
        let composed_dir = root.join("presets_composed");

        assert!(
            presets_dir.is_dir(),
            "expected checked-in presets dir at {}",
            presets_dir.display()
        );
        assert!(
            composed_dir.is_dir(),
            "expected checked-in presets_composed dir at {}",
            composed_dir.display()
        );

        let scan_dirs = vec![
            (presets_dir.to_str().unwrap(), false),
            (composed_dir.to_str().unwrap(), true),
        ];
        let collected = collect_presets(&scan_dirs, false, &all_gpus());

        let expected = count_json_files(&presets_dir) + count_json_files(&composed_dir);
        assert_eq!(
            collected.len(),
            expected,
            "collect_presets returned {} presets, but directory scan counted {}",
            collected.len(),
            expected
        );
    }

    #[test]
    fn collect_presets_marks_composed_directory_correctly() {
        let root = repo_presets_root();
        let composed_dir = root.join("presets_composed");

        if count_json_files(&composed_dir) == 0 {
            return;
        }

        let scan_dirs = vec![(composed_dir.to_str().unwrap(), true)];
        let collected = collect_presets(&scan_dirs, false, &all_gpus());

        assert!(!collected.is_empty());
        for p in &collected {
            assert!(
                p.is_composed,
                "preset {} should be flagged is_composed=true",
                p.name
            );
        }
    }

    #[test]
    fn collect_presets_gpu_filter_drops_hardware_specific_files() {
        let root = repo_presets_root();
        let presets_dir = root.join("presets");
        assert!(
            presets_dir.is_dir(),
            "expected checked-in presets dir at {}",
            presets_dir.display()
        );

        let unfiltered = collect_presets(&[(presets_dir.to_str().unwrap(), false)], false, &all_gpus());
        let filtered = collect_presets(&[(presets_dir.to_str().unwrap(), false)], true, &no_gpus());

        assert!(!unfiltered.is_empty(), "expected some unfiltered presets");

        assert!(
            filtered.len() <= unfiltered.len(),
            "filtered ({}) should be <= unfiltered ({})",
            filtered.len(),
            unfiltered.len()
        );

        for p in &filtered {
            assert!(
                !p.name.ends_with("-cuda"),
                "filtered presets must not include -cuda: {}",
                p.name
            );
            assert!(
                !p.name.ends_with("-rocm"),
                "filtered presets must not include -rocm: {}",
                p.name
            );
            assert!(
                !p.name.ends_with("-vulkan"),
                "filtered presets must not include -vulkan: {}",
                p.name
            );
        }
    }

    #[test]
    fn collect_presets_skips_non_json_files() {
        let root = repo_presets_root();
        let presets_dir = root.join("presets");

        let tmp = std::env::temp_dir().join(format!(
            "nix-helper-collect-presets-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("real.json"), r#"{"name":"x","display_name":"X","description":"d","url":"u","command":"c"}"#).unwrap();
        fs::write(tmp.join("notes.txt"), "not a preset").unwrap();
        fs::write(tmp.join("README.md"), "# readme").unwrap();

        let scan_dirs = vec![(tmp.to_str().unwrap(), false)];
        let collected = collect_presets(&scan_dirs, false, &all_gpus());

        assert_eq!(collected.len(), 1, "only the .json file should be collected");
        assert_eq!(collected[0].name, "x");

        let _ = fs::remove_dir_all(&tmp);
        let _ = presets_dir;
    }

    #[test]
    fn collect_presets_is_faster_than_serial_baseline() {
        use std::time::Instant;
        let root = repo_presets_root();
        let presets_dir = root.join("presets");
        let composed_dir = root.join("presets_composed");
        let scan_dirs = vec![
            (presets_dir.to_str().unwrap(), false),
            (composed_dir.to_str().unwrap(), true),
        ];

        let t0 = Instant::now();
        let par = collect_presets(&scan_dirs, false, &all_gpus());
        let par_elapsed = t0.elapsed();

        let t0 = Instant::now();
        let mut ser = Vec::new();
        for (dir, is_composed) in &scan_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(mut p) = serde_json::from_str::<PresetInfo>(&content) {
                                p.is_composed = *is_composed;
                                ser.push(p);
                            }
                        }
                    }
                }
            }
        }
        let ser_elapsed = t0.elapsed();

        eprintln!(
            "presets scan: parallel={:?} serial={:?} ({} presets)",
            par_elapsed, ser_elapsed, par.len()
        );

        assert_eq!(par.len(), ser.len(), "parallel and serial counts must match");
        assert_eq!(par.len(), count_json_files(&presets_dir) + count_json_files(&composed_dir));
    }
}