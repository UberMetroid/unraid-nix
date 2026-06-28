<?php if ($settings_confirmed !== 'yes'): ?>
    <div class="nix-section" style="margin-bottom: 20px; border-left: 4px solid #f39c12; background: rgba(243, 156, 18, 0.08); padding: 15px; border-radius: 4px;">
        <div style="display: flex; align-items: flex-start; gap: 12px;">
            <i class="fa fa-warning" style="color: #f39c12; font-size: 20px; margin-top: 2px;"></i>
            <div>
                <h4 style="margin: 0 0 5px 0; color: #f39c12; font-size: 14px;">Initial Setup Required</h4>
                <p class="nix-subtext" style="margin: 0; color: var(--nix-text-primary); font-size: 12px; line-height: 1.5;">
                    The Nix Package Manager is not initialized. Please verify your <strong>Storage Location</strong> under Configuration below (do <strong>NOT</strong> use your USB boot flash drive '/boot') and click <strong>Apply Configuration</strong> to start the Nix system daemon.
                </p>
            </div>
        </div>
    </div>
<?php endif; ?>

<!-- 1. Daemon -->
<div class="nix-section" style="margin-bottom: 20px;">
    <div style="display: flex; align-items: center; justify-content: space-between;">
        <div>
            <h3 style="margin: 0 0 5px 0;">Daemon</h3>
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

<!-- 2. Configuration -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3>Configuration</h3>
    <p class="nix-subtext">Define where packages are stored and how the environment boots.</p>
    
    <div class="nix-form-group">
        <label for="settings-store-path">Storage Location:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Target directory for package persistence. Do NOT use your USB boot drive (/boot). (Default: /mnt/cache/system/nix)</div>
        <div style="position: relative;">
            <input type="text" id="settings-store-path" autocomplete="off" spellcheck="false" value="<?php echo htmlspecialchars($store_path); ?>" placeholder="/mnt/cache/system/nix" data-pickcloseonfile="true" data-pickfilter="HIDE_FILES_FILTER" data-pickroot="/mnt" data-pickfolders="true" required style="padding-right: 30px;">
            <i class="fa fa-folder-open nix-folder-picker-btn" style="position: absolute; right: 10px; top: 50%; transform: translateY(-50%); color: var(--nix-text-muted); pointer-events: none;"></i>
        </div>
    </div>

    <div class="nix-form-group">
        <label for="settings-autostart">Autostart Services:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Automatically start all background services on system array startup. (Default: Yes)</div>
        <select id="settings-autostart">
            <option value="yes" <?php echo $autostart === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $autostart === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-show-in-nav">Show in Navigation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Show the Nix tab inside Unraid's top header navigation bar. (Default: Yes)</div>
        <select id="settings-show-in-nav">
            <option value="yes" <?php echo $show_in_nav === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $show_in_nav === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-nix-channel">Channel Pin:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">The repository branch mapping key to resolve package dependency versions. (Default: nixos-unstable)</div>
        <select id="settings-nix-channel">
            <option value="nixos-unstable" <?php echo $nix_channel === 'nixos-unstable' ? 'selected' : ''; ?>>nixos-unstable</option>
            <option value="nixos-24.05" <?php echo $nix_channel === 'nixos-24.05' ? 'selected' : ''; ?>>nixos-24.05 (Stable)</option>
            <option value="nixos-23.11" <?php echo $nix_channel === 'nixos-23.11' ? 'selected' : ''; ?>>nixos-23.11 (Stable)</option>
        </select>
    </div>

    <div style="margin-top: 15px; display: flex; justify-content: flex-end; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <button type="button" class="nix-btn-primary" style="margin: 0; padding: 6px 18px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px;" onclick="saveAllSettings(this)"><i class="fa fa-check"></i> Apply Configuration</button>
    </div>
</div>

<!-- 3. Flakes -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3>Flakes</h3>
    <p class="nix-subtext">Fine-tune build constraints and package display filters.</p>

    <div class="nix-form-group">
        <label for="settings-allow-source-builds">Local Flake Compilation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Allow compilation of packages locally from source code when binary cache is missing. (Default: No)</div>
        <select id="settings-allow-source-builds">
            <option value="yes" <?php echo $allow_source_builds === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $allow_source_builds === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-build-cores">Max Build Cores:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Maximum CPU threads Nix can consume per build job. Set to 0 to use all available cores. (Default: 0)</div>
        <input type="number" id="settings-build-cores" min="0" value="<?php echo htmlspecialchars($build_cores); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div class="nix-form-group">
        <label for="settings-build-jobs">Max Build Jobs:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Maximum concurrent compilation jobs Nix can execute in parallel. Set to 0 to auto-assign. (Default: 0)</div>
        <input type="number" id="settings-build-jobs" min="0" value="<?php echo htmlspecialchars($build_jobs); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div class="nix-form-group">
        <label for="settings-filter-presets-by-hardware">GPU Preset Filtering:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Filter preset templates grid based on host GPU hardware compatibility. (Default: Yes)</div>
        <select id="settings-filter-presets-by-hardware">
            <option value="yes" <?php echo $filter_presets_by_hardware === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $filter_presets_by_hardware === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div style="margin-top: 15px; display: flex; justify-content: flex-end; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <button type="button" class="nix-btn-primary" style="margin: 0; padding: 6px 18px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px;" onclick="saveAllSettings(this)"><i class="fa fa-check"></i> Apply Flakes</button>
    </div>
</div>

<!-- 4. Isolation -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3>Isolation</h3>
    <p class="nix-subtext">Restrict namespace access to separate guest services from the host.</p>
    
    <div class="nix-form-group">
        <label for="settings-enable-sandbox">Storage Isolation (jail):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Run guest services inside a private chroot jail restricting host filesystem visibility. (Default: No)</div>
        <select id="settings-enable-sandbox">
            <option value="yes" <?php echo $enable_sandbox === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_sandbox === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-enable-pid-isolation">Process (PID) Isolation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Hide other host OS processes from the guest service list. (Default: Yes)</div>
        <select id="settings-enable-pid-isolation">
            <option value="yes" <?php echo $enable_pid_isolation === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_pid_isolation === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-enable-uts-isolation">Hostname (UTS) Isolation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Expose a virtual sandboxed hostname to running guest services. (Default: Yes)</div>
        <select id="settings-enable-uts-isolation">
            <option value="yes" <?php echo $enable_uts_isolation === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_uts_isolation === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group">
        <label for="settings-enable-ipc-isolation">IPC Namespace Isolation:</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Isolate inter-process communication resources and message queues. (Default: Yes)</div>
        <select id="settings-enable-ipc-isolation">
            <option value="yes" <?php echo $enable_ipc_isolation === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $enable_ipc_isolation === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div style="margin-top: 15px; display: flex; justify-content: flex-end; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <button type="button" class="nix-btn-primary" style="margin: 0; padding: 6px 18px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px;" onclick="saveAllSettings(this)"><i class="fa fa-check"></i> Apply Isolation</button>
    </div>
</div>

<!-- 5. Maintenance -->
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
        <label for="settings-auto-gc">Weekly Cleanup (GC):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Automatically purge dangling build inputs weekly, mirroring Unraid's scheduler. (Default: No)</div>
        <select id="settings-auto-gc">
            <option value="yes" <?php echo $auto_gc === 'yes' ? 'selected' : ''; ?>>Yes</option>
            <option value="no" <?php echo $auto_gc === 'no' ? 'selected' : ''; ?>>No</option>
        </select>
    </div>

    <div class="nix-form-group" style="margin-top: 15px;">
        <label for="settings-store-quota">Storage Quota (GB):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Target size threshold. Background cleanup tasks run GC automatically if exceeded. (Default: 30)</div>
        <div style="display: flex; align-items: center; gap: 10px;">
            <input type="number" id="settings-store-quota" min="10" value="<?php echo htmlspecialchars($store_quota); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
            <span style="font-size: 12px; color: var(--nix-text-secondary);">
                Suggested: <strong><?php echo $suggested_quota; ?> GB</strong> (based on <?php echo $num_flakes; ?> configured flakes)
            </span>
        </div>
    </div>

    <div class="nix-form-group" style="margin-top: 15px; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <label for="settings-gc-min-free">Min Free Disk Space (GB):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Automatically trigger GC if your Unraid storage pool free space falls below this limit. (Default: 5)</div>
        <input type="number" id="settings-gc-min-free" min="1" value="<?php echo htmlspecialchars($gc_min_free); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div class="nix-form-group" style="margin-top: 15px;">
        <label for="settings-gc-max-free">GC Cleanup Amount (GB):</label>
        <div class="nix-field-help" style="margin-top: 0; margin-bottom: 6px;">Amount of storage space Nix should clear out when disk-space-triggered GC fires. (Default: 10)</div>
        <input type="number" id="settings-gc-max-free" min="1" value="<?php echo htmlspecialchars($gc_max_free); ?>" style="width: 100px; padding: 6px; border-radius: 4px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-secondary); color: var(--nix-text-primary); text-align: center;">
    </div>

    <div style="margin-top: 15px; display: flex; justify-content: flex-end; border-top: 1px solid rgba(255,255,255,0.05); padding-top: 15px;">
        <button type="button" class="nix-btn-primary" style="margin: 0; padding: 6px 18px; font-size: 12px; display: inline-flex; align-items: center; gap: 6px;" onclick="saveAllSettings(this)"><i class="fa fa-check"></i> Apply Maintenance</button>
    </div>
</div>

<!-- 6. Hardware Capabilities -->
<div class="nix-section" style="margin-bottom: 20px;">
    <h3 style="margin: 0 0 10px 0;">Hardware Capabilities</h3>
    <div style="display: flex; gap: 15px; flex-wrap: wrap;">
        <div style="flex: 1; min-width: 240px; padding: 12px; background: var(--nix-bg-secondary); border-radius: 6px; border: 1px solid var(--nix-border-primary);">
            <div style="font-size: 10px; font-weight: bold; text-transform: uppercase; color: var(--nix-text-muted); margin-bottom: 6px; letter-spacing: 0.5px;">CPU Virtualization (KVM)</div>
            <div style="display: flex; align-items: center; gap: 8px;">
                <?php if ($has_kvm): ?>
                    <i class="fa fa-check-circle" style="color: #2ecc71; font-size: 16px;"></i>
                    <span style="font-size: 13px; font-weight: bold; color: var(--nix-text-primary);">Active (SVM/VT-x Enabled)</span>
                <?php else: ?>
                    <i class="fa fa-warning" style="color: #e67e22; font-size: 16px;"></i>
                    <span style="font-size: 13px; font-weight: bold; color: #e67e22;">Not Enabled (Virtualization disabled in BIOS)</span>
                <?php endif; ?>
            </div>
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
