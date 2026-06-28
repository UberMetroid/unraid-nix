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