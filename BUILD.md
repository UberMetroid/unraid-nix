# Building & Releasing nix-helper

The compiled `nix-helper` ELF binary is **not** committed to this
repository. It is built either locally for development or by CI on tag
push and attached to the matching [GitHub Release](https://github.com/UberMetroid/unraid-nix/releases).

This keeps `git clone` fast (the binary is 4 MB; the source is ~30 KB)
and ensures users install a known, reproducible artifact.

---

## Prerequisites

* Rust stable (1.75+ recommended; `cargo --version` to check)
* On Debian/Ubuntu: `apt install build-essential pkg-config libssl-dev`
* `gh` CLI for releases (`apt install gh`, then `gh auth login`)

---

## Local development build

```bash
./scripts/build.sh           # debug build, large binary, fast compile
./scripts/build.sh --release # release build, stripped + LTO, ~1 MB
```

The binary is placed at the repo root as `./nix-helper` (matching what
`nix.plg` references during install). It is gitignored.

---

## Running the tests

```bash
cargo test
```

> **Note:** this is `cargo test`, **not** `cargo test --workspace`. The
> plugin is a single binary crate (`nix-helper`) at the repository root; the
> `--workspace` flag is unnecessary and will fail if a top-level `Cargo.toml`
> is later moved into a workspace root.

### Test coverage

The unit test suite (51 tests as of the most recent sweep) covers:

* **Config parser/serializer** (`src/config/mod.rs`, `src/config/file.rs`):
  process-compose YAML round-trip, default settings initialization,
  process definition sanity.
* **Service command presets** (`src/config/mod.rs`): assert that
  `get_service_command_preset` produces the expected `unshare`, mount,
  `setpriv`, and `sed` substitutions for Radarr, Jellyfin, and unknown
  services (rejection path).
* **Storage sandbox builder** (`src/sandbox/mod.rs`,
  `src/sandbox/builder_tests.rs`, `src/sandbox/cli.rs`): verify that the
  generated `bwrap` / `unshare -m` command exposes only the requested
  bind-mounts, includes the GPU stub when GPU access is requested, and
  rejects store-on-`/boot` paths.
* **Sandbox CLI argument parser** (`src/sandbox/cli.rs`): every public
  flag of `nix-helper sandbox …` produces the right inner argv under
  representative invocations.
* **Process / supervisor parser** (`src/process/status.rs`,
  `src/process/ports.rs`): uptime, port, and resource-extraction
  helpers used by the dashboard renderer.
* **Logs CLI** (`src/cli/logs.rs`): log-level filtering, sanitisation
  of log-injection metacharacters, and tail-window clamping.
* **Settings CLI** (`src/cli/settings/mod.rs`): `save-settings`
  argument assembly for every boolean flag and numeric bound.
* **Unraid share detection** (`src/unraid.rs`): parsing of
  `shares/*.cfg`, cache-pool selection, and default-store-path
  inference.
* **Search JSON mapper** (`src/search/mod.rs`, `src/search/parser.rs`):
  Nix search.nixos.org result normalization.
* **Store account bootstrap** (`src/store/accounts.rs`,
  `src/store/config/tests.rs`): user/group creation, sanitisation of
  arbitrary service names, and log-rotation defaults.
* **Preset store renderer** (`src/api/presets_store/mod.rs`): category
  bucketing and HTML escape behaviour for service descriptions.
* **Services row formatter** (`src/api/services_row/row_formatter/mod.rs`):
  status badge, version badge, and resource cell HTML escaping for
  arbitrary service names.
* **API helpers** (`src/api/mod.rs`): version-header construction for
  the `get_dashboard_json` endpoint.

---

## Cutting a release

```bash
./scripts/release.sh              # auto-bumps patch version
./scripts/release.sh 2026.07.01.1  # explicit version
```

The script:

1. Bumps the version in `nix.plg` (`<!ENTITY version "...">`).
2. Builds the release binary.
3. Commits, tags (`v<version>`), and pushes to `origin/main`.
4. Triggers `.github/workflows/release.yml`, which builds the binary
   from CI and attaches it to the GitHub Release.

The local script also uploads a `nix-helper` asset to the release for
convenience; the CI-built artifact is the canonical one and will
overwrite it if both exist.

---

## Why a separate workflow + local script?

* **CI is canonical.** A build from a developer's machine may have
  hidden toolchain differences; CI is reproducible.
* **Local script is convenient.** When cutting a release you want the
  artifact immediately available without waiting for CI.
* **`nix.plg` is the source of truth for the version.** Both the local
  script and CI read it; never hard-code the version anywhere else.

---

## Future work

* Cross-compile to `aarch64-unknown-linux-gnu` (Unraid on ARM NAS).
* Use `cargo dist` instead of hand-rolled shell.
* Sign the binary with `cosign` for supply-chain integrity.