# unraid-nix Feature Roadmap & TODO List

Below is the structured roadmap of features to be added to the `unraid-nix` plugin project:

---

## 1. Upgrade & Maintenance Controls
- [x] **Global Flake Update Scanner**:
  - Implement a check in the backend to determine if installed flakes have updates available from the Nix registry/channel.
  - Display a visual indicator/count of packages requiring updates.
- [x] **Global Rebuild**:
  - Add a "Rebuild All Services" button that triggers a batch update and recompilation of all configured flakes.

## 2. Subsystem Resource Dashboard
- [ ] **Combined Subsystem Sparkline**:
  - Place a dashboard header at the top of the **Flakes** tab.
  - Render a real-time sparkline graph showing combined CPU, RAM, and GPU utilization for all running Nix flakes combined.

## 3. Flakes Tab Card Layout
- [ ] **Adaptive Flake Cards**:
  - Refactor the current tabular services list into a grid of status cards, matching the size, format, and layout style of the Template Store.
- [ ] **Expandable Details / Modals**:
  - Provide an inline "Expand" toggle or a popup modal dialog for each card to show detailed stats (individual ports, exact bind-mounts, resource graphs, and service status logs) without cluttering the main grid view.

## 4. Onboarding & Tour Guide
- [ ] **First-Time Setup Walkthrough**:
  - Implement a modal dialog/popup that triggers automatically for new installations.
  - Guide the user through selecting a storage location, configuring sandboxing defaults, and setting initial CPU core quotas.
- [ ] **Interactive Tour**:
  - Provide a guided visual tour highlighting the key tabs (Flakes, Templates, Settings, Logs).

## 5. Security & Isolation Tester
- [ ] **Internal Isolation Tester Flake**:
  - Create a lightweight test flake packaged with check scripts.
  - When deployed, it runs tests *from the inside* of the chroot jail to verify isolation boundaries (checking if host `/etc/shadow` is hidden, host PIDs are invisible, and host IPC namespaces are isolated) and produces a security report.

## 6. Diagnostic Logging & Troubleshooting
- [x] **One-Click Diagnostic Bundle**:
  - Add a button in the **Logs** or **Settings** tab to zip and download all nix process logs, system startup logs, and config files.
- [x] **GitHub Issues Helper**:
  - Provide clear instructions and links to easily post bug reports and upload the diagnostic zip directly to [UberMetroid/unraid-nix/issues](https://github.com/UberMetroid/unraid-nix/issues).

## 7. Tunnels & Network Sandbox Routing
- [ ] **Unified Network Tunnels Manager**:
  - Add a dedicated **Tunnels** tab or settings section to configure secure routing tunnels:
    - **Wireguard / OpenVPN**: Manage client configs (like PIA, Mullvad) in custom network namespaces (`netns`).
    - **Cloudflare Tunnels**: Manage `cloudflared` tokens, monitor tunnel health, and auto-detect domain mapping setups.
    - **Tailscale / Tailscale Funnel**: Add Tailscale client connections, tailscale IPs, and reverse funnel links.
- [ ] **Secure Credentials Vault**:
  - Create a secure settings location for API keys (e.g., Cloudflare API tokens, Tailscale auth keys) to automatically fetch tunnel mappings and status.
- [ ] **Per-Flake Routing Selection**:
  - Add a dropdown to each flake's edit config screen to choose its default network gateway (e.g., Host LAN, specific VPN netns, or Loopback Only).

## 8. Core Architecture Migrations
- [x] **Port GPU Setup script to Rust**:
  - Expose CUDA/NVIDIA driver symlinks natively via `nix-helper setup-gpus`.
- [x] **Port Installer SSE stream to Rust**:
  - Capture stdout/stderr in real-time and tail logs natively via `nix-helper stream-install`.
- [x] **Type-safe CLI Parser**:
  - Replace hand-rolled dispatcher with standard `clap` modules.
- [x] **GPU Discovery Caching**:
  - Add RAM tmpfs cache for settings/presets scans and bypass `nvidia-smi` on non-Nvidia hosts.
- [x] **Hardened Chroot Sandboxing**:
  - Implement read-only mounts overlay and proper isolated `procfs` namespaces.
