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
- [ ] **GitHub Issues Helper (automated upload)**:
  - Manual upload instructions and a "GitHub Issues" link exist on the **Logs** tab; the original spec called for one-click upload of the diagnostic tarball straight into a new GitHub Issue. That flow requires an OAuth token / GitHub App registration that has not been implemented yet. The diagnostic bundle is currently downloaded locally and the user attaches it manually.

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

---

## 9. Backlog (recent security & quality sweep)

Items below were surfaced during the recent security/quality review. None
are critical, but they should be addressed before the next release.

### Security defense-in-depth
- [ ] **CSP / `X-Content-Type-Options: nosniff` on remaining HTML endpoints**
  The `Logs` page now emits a `Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'` meta tag and the `api.php` HTML routes (`logs`, `search`, `render-services`, `render-presets`, `get_dashboard`) emit `X-Content-Type-Options: nosniff`. The remaining `.page` files (`nix_settings`, `nix_install`, `nix_registry`, `nix_search`, `nix_services`) still rely on Unraid's default headers; add the same meta tag (or wrap the page in a helper that emits the header before any output) to cover them.
- [ ] **Drop `serde_yaml = "0.9"` (deprecated crate)** — DONE (now on `serde_yml = "0.0.12"`). Re-evaluate after `serde_yml` publishes a non-deprecation-warning release or migrate the small config round-trip to `serde_yaml_ng` once stable.
- [ ] **Diagnostic bundle permissions** — DONE (tmp dir `0700`, all `copy`/`file_put_contents` outputs `chmod 0600` before packaging).
- [ ] **JS `innerHTML` for user-controlled data** — Remaining uses (`stream.js`, `nix.js`, `nix_install.js`, `nix_install_presets.js`, `nix_services_updates.js`) all operate on server-rendered or hard-coded content; none currently interpolate user-supplied strings. Add a lint rule (or pre-commit hook) to flag future regressions where untrusted data flows into `.html()` / `.innerHTML`.

### Quality / consistency
- [ ] **`presets/` count drift** — `nix.plg` and `README.md` previously claimed "220+" / "165+" presets; the actual count is 167 JSON files. Both references have been rephrased to a count-agnostic statement. Re-run `ls presets/*.json | wc -l` whenever bumping the templates repo.
- [ ] **Audit unused CSS selectors** — `styles_cards.css` / `styles_layout.css` are small enough that dead selectors are tolerable, but should be revisited the next time the stylesheets change significantly.
- [ ] **Tests documented in `BUILD.md`** — DONE; the "Test coverage" section enumerates the 51 unit tests grouped by module.

### Operational
- [ ] **GitHub Issues automated upload** (see Section 6) — still unimplemented.
- [ ] **CSRF token bootstrap on pages that submit POST without one** — Currently `window.csrf_token` is read from `nix.js` which sets it during page load; if a page is opened before `nix.js` finishes loading the request can race. Consider injecting the token via a `<meta>` tag so it is available synchronously.
- [ ] **`stream_init.php` rejects `LD_PRELOAD` only by prefix** — env-var sanitiser currently only rejects keys starting with `LD_`. Consider rejecting any key containing `LD_` (e.g. `PATH_LD_PRELOAD`) for defence in depth.
- [ ] **`nix_logs.page` service log globbing** — currently `glob('/var/log/nix-services/*.log')`. The log name is regex-validated before being interpolated; this is correct but a comment explaining the defence would help future readers.
