<?php if ($settings_confirmed !== 'yes'): ?>
    <div class="nix-section" style="margin-bottom: 20px; border-left: 4px solid #f39c12; background: rgba(243, 156, 18, 0.08); padding: 15px; border-radius: 4px;">
        <div style="display: flex; align-items: flex-start; gap: 12px;">
            <i class="fa fa-warning" style="color: #f39c12; font-size: 20px; margin-top: 2px;"></i>
            <div>
                <h4 style="margin: 0 0 5px 0; color: #f39c12; font-size: 14px;">Initial Setup Required</h4>
                <p class="nix-subtext" style="margin: 0; color: var(--nix-text-primary); font-size: 12px; line-height: 1.5;">
                    The Nix Package Manager is not initialized. Please verify your <strong>Storage Location</strong> under Configuration below (do <strong>NOT</strong> use your USB boot flash drive '/boot') and click <strong>Apply Settings</strong> at the bottom of the page to start the Nix system daemon.
                </p>
            </div>
        </div>
    </div>
<?php endif; ?>

<!-- 1. Daemon Status & Controls -->
<div class="nix-section" style="margin-bottom: 20px;">
    <div style="display: flex; align-items: center; justify-content: space-between;">
        <div>
            <h3 style="margin: 0 0 5px 0;">Daemon Status & Controls</h3>
            <p class="nix-subtext" style="margin: 0;">
                Status: 
                <?php if ($is_nix_running): ?>
                    <span class="status green">RUNNING</span>
                <?php else: ?>
                    <span class="status red">STOPPED</span>
                <?php endif; ?>
            </p>
        </div>
        <div id="nix-daemon-controls" style="display: flex; gap: 8px;">
            <?php if ($is_nix_running): ?>
                <button type="button" class="nix-btn-install" style="background: var(--nix-bg-tertiary); border: 1px solid var(--nix-border-primary); color: var(--nix-text-muted); cursor: not-allowed; font-weight: normal; margin: 0; padding: 6px 12px; display: inline-flex; align-items: center; gap: 6px;" disabled><i class="fa fa-play"></i> Start</button>
                <button type="button" class="nix-btn" style="margin: 0; padding: 6px 12px;" onclick="toggleNixDaemon('stop')"><i class="fa fa-stop"></i> Stop</button>
                <button type="button" class="nix-btn" style="margin: 0; padding: 6px 12px;" onclick="toggleNixDaemon('restart')"><i class="fa fa-refresh"></i> Restart</button>
            <?php else: ?>
                <button type="button" class="nix-btn-install" style="margin: 0; padding: 6px 12px; display: inline-flex; align-items: center; gap: 6px;" onclick="toggleNixDaemon('start')"><i class="fa fa-play"></i> Start</button>
                <button type="button" class="nix-btn" style="margin: 0; padding: 6px 12px; color: var(--nix-text-muted); border-color: var(--nix-border-primary); cursor: not-allowed;" disabled><i class="fa fa-stop"></i> Stop</button>
                <button type="button" class="nix-btn" style="margin: 0; padding: 6px 12px; color: var(--nix-text-muted); border-color: var(--nix-border-primary); cursor: not-allowed;" disabled><i class="fa fa-refresh"></i> Restart</button>
            <?php endif; ?>
        </div>
    </div>
</div>

<!-- 2. System Paths & Integration -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3>System Paths & Integration</h3>
    <p class="nix-subtext">Configure where data is stored and how Nix integrates with the Unraid WebUI.</p>
    
    <div class="nix-form-group">
        <label for="settings-store-path">Nix Store Path:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Select the persistent storage directory on your array or cache pool where the package store (/nix) resides. Avoid using your USB flash boot drive (/boot).</div>
        <div style="position: relative;">
            <input type="text" id="settings-store-path" autocomplete="off" spellcheck="false" value="<?php echo htmlspecialchars($store_path); ?>" placeholder="/mnt/cache/system/nix" data-pickcloseonfile="true" data-pickfilter="HIDE_FILES_FILTER" data-pickroot="/mnt" data-pickfolders="true" required style="padding-right: 30px;">
            <i class="fa fa-folder-open nix-folder-picker-btn" style="position: absolute; right: 10px; top: 50%; transform: translateY(-50%); color: var(--nix-text-muted); pointer-events: none;"></i>
        </div>
    </div>

    <div class="nix-form-group">
        <label for="settings-default-appdata-path">Default Appdata Path:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Select the default base directory where configuration folders (/config) for new guest services will be created.</div>
        <div style="position: relative;">
            <input type="text" id="settings-default-appdata-path" autocomplete="off" spellcheck="false" value="<?php echo htmlspecialchars($default_appdata_path); ?>" placeholder="/mnt/user/appdata" data-pickcloseonfile="true" data-pickfilter="HIDE_FILES_FILTER" data-pickroot="/mnt" data-pickfolders="true" required style="padding-right: 30px;">
            <i class="fa fa-folder-open nix-folder-picker-btn" style="position: absolute; right: 10px; top: 50%; transform: translateY(-50%); color: var(--nix-text-muted); pointer-events: none;"></i>
        </div>
    </div>

    <div class="nix-form-group">
        <label for="settings-autostart">Autostart Flake Services:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Choose whether guest application services should automatically start up when the Unraid array mounts.</div>
        <select id="settings-autostart">
            <option value="yes" <?php echo $autostart === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $autostart === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-show-in-nav">Show in Navigation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Choose whether to display the Nix plugin tab in the top header navigation bar of Unraid.</div>
        <select id="settings-show-in-nav">
            <option value="yes" <?php echo $show_in_nav === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $show_in_nav === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>
</div>

<!-- 3. Package Manager & Channels -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3>Package Manager & Channels</h3>
    <p class="nix-subtext">Configure compilation policies, channel version mappings, and template catalog filtering.</p>

    <div class="nix-form-group">
        <label for="settings-nix-channel">Nix Channel Pin:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Select the default Nix channel to resolve and fetch packages (stable or unstable releases).</div>
        <select id="settings-nix-channel">
            <option value="nixos-unstable" <?php echo $nix_channel === 'nixos-unstable' ? 'selected' : ''; ?>>nixos-unstable (Recommended)</option>
            <option value="nixos-25.11" <?php echo $nix_channel === 'nixos-25.11' ? 'selected' : ''; ?>>nixos-25.11 (Stable)</option>
            <option value="nixos-25.05" <?php echo $nix_channel === 'nixos-25.05' ? 'selected' : ''; ?>>nixos-25.05 (Stable)</option>
            <option value="nixos-24.11" <?php echo $nix_channel === 'nixos-24.11' ? 'selected' : ''; ?>>nixos-24.11 (Stable)</option>
            <option value="nixos-24.05" <?php echo $nix_channel === 'nixos-24.05' ? 'selected' : ''; ?>>nixos-24.05 (Stable)</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-allow-source-builds">Allow Source Compilation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Allow Nix to compile custom source packages locally if pre-built binaries are not available in binary caches. Disabling this safeguards host resources.</div>
        <select id="settings-allow-source-builds">
            <option value="yes" <?php echo $allow_source_builds === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $allow_source_builds === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-build-cores">Max Cores per Job:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Restrict the maximum number of CPU threads Nix can consume per build job (set to 0 for unlimited). Only applies to local compilation.</div>
        <input type="number" id="settings-build-cores" min="0" value="<?php echo htmlspecialchars($build_cores); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div class="nix-form-group">
        <label for="settings-build-jobs">Max Concurrent Jobs:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Limit the maximum number of parallel compilation tasks Nix can execute at once (set to 0 for auto-assignment). Only applies to local compilation.</div>
        <input type="number" id="settings-build-jobs" min="0" value="<?php echo htmlspecialchars($build_jobs); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div class="nix-form-group">
        <label for="settings-filter-presets-by-hardware">GPU Preset Filtering:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Automatically filter the Flake Store grid to only display application templates matching compatible host GPU hardware.</div>
        <select id="settings-filter-presets-by-hardware">
            <option value="yes" <?php echo $filter_presets_by_hardware === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $filter_presets_by_hardware === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>
</div>

<!-- 4. Sandbox & Isolation -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3>Sandbox & Isolation</h3>
    <p class="nix-subtext">Configure secure namespace boundaries separating guest services from the host OS.</p>
    
    <div class="nix-form-group">
        <label for="settings-enable-sandbox">Sandbox Jail Isolation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Run guest services inside a secure, private chroot jail, restricting access to other host folders.</div>
        <select id="settings-enable-sandbox">
            <option value="yes" <?php echo $enable_sandbox === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_sandbox === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-enable-pid-isolation">Process Tree Isolation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Isolate process namespaces, hiding host processes and other guest tasks from the running service.</div>
        <select id="settings-enable-pid-isolation">
            <option value="yes" <?php echo $enable_pid_isolation === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_pid_isolation === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-enable-uts-isolation">Hostname Sandboxing:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Expose a virtual, sandboxed hostname to the guest service instead of sharing the Unraid host's name.</div>
        <select id="settings-enable-uts-isolation">
            <option value="yes" <?php echo $enable_uts_isolation === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_uts_isolation === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-enable-ipc-isolation">IPC Namespace Isolation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Isolate inter-process communication resources, preventing the service from sharing host memory queues.</div>
        <select id="settings-enable-ipc-isolation">
            <option value="yes" <?php echo $enable_ipc_isolation === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_ipc_isolation === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>
</div>

<!-- 5. Storage Maintenance & GC -->
<div class="nix-section" style="margin-bottom: 20px;">
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 15px; border-bottom: 1px solid rgba(255,255,255,0.05); padding-bottom: 15px;">
        <div>
            <h3 style="margin: 0 0 5px 0;">Template Library</h3>
            <p class="nix-subtext" style="margin: 0;">Sync and update self-hosted app templates from the remote repository.</p>
        </div>
        <div>
            <button type="button" id="nix-sync-btn" class="nix-btn" style="display: inline-flex; align-items: center; gap: 6px; margin: 0;" onclick="syncTemplates(this)"><i class="fa fa-refresh"></i> Force Sync</button>
        </div>
    </div>

    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 15px;">
        <div>
            <h3 style="margin: 0 0 5px 0;">Garbage Collection</h3>
            <p class="nix-subtext" style="margin: 0;">Clean up unused packages and reclaim storage pool space.</p>
        </div>
        <div style="text-align: right;">
            <button type="button" id="nix-gc-btn" class="nix-btn" style="display: inline-flex; align-items: center; gap: 6px; margin: 0;" onclick="collectGarbage(this)"><i class="fa fa-trash"></i> Collect Garbage</button>
        </div>
    </div>
    
    <div class="nix-form-group" style="margin-top: 15px; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <label for="settings-auto-gc">Weekly Garbage Collection:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Schedule a weekly background task to automatically purge dangling package dependencies and build tools.</div>
        <select id="settings-auto-gc">
            <option value="yes" <?php echo $auto_gc === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $auto_gc === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group" style="margin-top: 15px;">
        <label for="settings-store-quota">Store Size Quota (GB):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Set the target Nix store size limit. GC runs automatically in the background if the store size exceeds this limit.</div>
        <div style="display: flex; align-items: center; gap: 10px;">
            <input type="number" id="settings-store-quota" min="10" value="<?php echo htmlspecialchars($store_quota); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
            <span style="font-size: 12px; color: var(--nix-text-secondary);">
                Suggested: <strong><?php echo $suggested_quota; ?> GB</strong> (based on <?php echo $num_flakes; ?> configured flakes)
            </span>
        </div>
    </div>

    <div class="nix-form-group" style="margin-top: 15px; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <label for="settings-gc-min-free">Min Free Pool Space (GB):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Automatically trigger garbage collection if the free space on the host storage pool falls below this threshold.</div>
        <input type="number" id="settings-gc-min-free" min="1" value="<?php echo htmlspecialchars($gc_min_free); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div class="nix-form-group" style="margin-top: 15px;">
        <label for="settings-gc-max-free">GC Purge Target (GB):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">The target amount of disk space to reclaim when garbage collection is triggered by low disk space.</div>
        <input type="number" id="settings-gc-max-free" min="1" value="<?php echo htmlspecialchars($gc_max_free); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>
</div>

<!-- Unified Submit Action Bar -->
<div style="margin: 20px 0; display: flex; justify-content: flex-end; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
    <button type="button" class="nix-btn-primary" style="margin: 0; padding: 10px 24px; font-size: 13px; display: inline-flex; align-items: center; gap: 8px; font-weight: 600;" onclick="saveAllSettings(this)"><i class="fa fa-check"></i> Apply Settings</button>
</div>

<!-- 6. Hardware Capability Diagnostics -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3 style="margin: 0 0 10px 0;">Hardware Capability Diagnostics</h3>
    <div style="display: flex; gap: 15px; flex-wrap: wrap;">
        <div style="flex: 1; min-width: 240px; padding: 12px; background: var(--nix-bg-secondary); border-radius: 6px; border: 1px solid var(--nix-border-primary);">
            <div style="font-size: 10px; font-weight: bold; text-transform: uppercase; color: var(--nix-text-muted); margin-bottom: 6px; letter-spacing: 0.5px;">CPU Virtualization (KVM)</div>
            <div style="display: flex; align-items: center; gap: 8px;">
                <?php if ($kvm_status === 'active'): ?>
                    <i class="fa fa-check-circle" style="color: #2ecc71; font-size: 16px;"></i>
                    <span style="font-size: 13px; font-weight: bold; color: var(--nix-text-primary);"><?php echo htmlspecialchars($kvm_details); ?></span>
                <?php elseif ($kvm_status === 'disabled'): ?>
                    <i class="fa fa-warning" style="color: #e67e22; font-size: 16px;"></i>
                    <span style="font-size: 13px; font-weight: bold; color: #e67e22;"><?php echo htmlspecialchars($kvm_details); ?></span>
                <?php else: ?>
                    <i class="fa fa-times-circle" style="color: var(--nix-text-muted); font-size: 16px;"></i>
                    <span style="font-size: 13px; font-weight: bold; color: var(--nix-text-muted);"><?php echo htmlspecialchars($kvm_details); ?></span>
                <?php endif; ?>
            </div>
            <?php if ($kvm_status === 'active' && !empty($kvm_features)): ?>
                <div style="margin-top: 10px; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 8px; display: flex; flex-direction: column; gap: 4px; font-size: 11px;">
                    <div style="display: flex; justify-content: space-between;">
                        <span style="color: var(--nix-text-muted);">Nested Virtualization:</span>
                        <span style="color: <?php echo $kvm_features['nested'] === 'Enabled' ? '#2ecc71' : 'var(--nix-text-muted)'; ?>; font-weight: bold;"><?php echo $kvm_features['nested']; ?></span>
                    </div>
                    <div style="display: flex; justify-content: space-between;">
                        <span style="color: var(--nix-text-muted);">IOMMU (VT-d/AMD-Vi):</span>
                        <span style="color: <?php echo $kvm_features['iommu'] === 'Enabled (IOMMU active)' ? '#2ecc71' : '#e67e22'; ?>; font-weight: bold;"><?php echo $kvm_features['iommu']; ?></span>
                    </div>
                    <div style="display: flex; justify-content: space-between;">
                        <span style="color: var(--nix-text-muted);">SLAT (EPT/RVI):</span>
                        <span style="color: var(--nix-text-primary); font-weight: bold;"><?php echo $kvm_features['slat']; ?></span>
                    </div>
                </div>
            <?php endif; ?>
        </div>
        <div style="flex: 1; min-width: 280px; padding: 12px; background: var(--nix-bg-secondary); border-radius: 6px; border: 1px solid var(--nix-border-primary);">
            <div style="font-size: 10px; font-weight: bold; text-transform: uppercase; color: var(--nix-text-muted); margin-bottom: 6px; letter-spacing: 0.5px;">GPU Transcoding & Passthrough</div>
            <div style="font-size: 13px; font-weight: bold; color: var(--nix-text-primary); display: flex; flex-direction: column; gap: 4px;">
                <?php foreach ($gpu_info as $gpu): ?>
                    <div style="display: flex; align-items: center; gap: 8px;">
                        <i class="fa fa-desktop" style="color: var(--nix-accent); font-size: 14px;"></i>
                        <span><?php echo htmlspecialchars($gpu); ?></span>
                    </div>
                <?php endforeach; ?>
            </div>
        </div>
    </div>
</div>
