# Project Status Update

**Date**: June 27, 2026

We have completed the major compiled Rust migrations, CLI modernization, and sandbox hardening. Below is the updated state of the `unraid-nix` plugin project:

---

## 1. Accomplished Features & Refactoring

* **GPU Hardware Acceleration & Passthrough (Rust Port)**:
  * Ported the legacy `web/nix-gpu-setup.sh` script to a native Rust command `nix-helper setup-gpus`.
  * Employs tmpfs RAM-disk caching (`/var/run/nix-detected-gpus.json`) to store GPU scan results across settings pages and template browsing, reducing process-spawning overhead.
  * Added hardware check bypasses in background telemetry loops (`get_gpu_active_services()` and `get_nvidia_pmon_stats()`) to completely prevent running `nvidia-smi` on hosts without NVIDIA GPUs.
* **Installer Output Stream (Rust Port)**:
  * Ported the installer process streaming loop and log tailing from `web/stream.php` to a native Rust command `nix-helper stream-install` and `nix-helper view-logs`.
  * Trimmed `web/stream.php` and `web/stream_init.php` to act as simple pass-through scripts, keeping all files well under the 250-line limit.
* **CLI Modernization via Clap**:
  * Migrated the hand-rolled dispatcher to a type-safe `clap` enum parser inside `src/cli/args.rs`.
  * Provides nested subcommands support, command validation, and automatic help flag outputs.
* **Hardened Sandbox Isolations**:
  * Configured all chroot sandbox directories to mount host files (`/etc/passwd`, `/etc/group`, `/etc/hosts`, `/etc/resolv.conf`, `/etc/ssl`, `/etc/nix` and `/nix`) as **read-only** (`-o ro`).
  * Replaced recursive procfs binds with a clean isolated mount (`mount -t proc proc`) inside the namespaces.
  * Added thread-local override states in tests to prevent parallel test runner environment races.

---

## 2. Next Steps / Current Action Item

1. **Telemetry Sparklines & Graph Integrations**: Integrate combined CPU/RAM/GPU telemetry metrics graphs at the top of the Flakes tab.
2. **Onboarding & Tour Guide**: Design a visual walkthrough modal for first-time plugin installs.
