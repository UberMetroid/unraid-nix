# Project Status Update

**Date**: June 26, 2026

We have resolved the Jellyfin transcoding and GPU passthrough issue and pushed it to GitHub. Below is the updated state of the `unraid-nix` plugin project:

---

## 1. Accomplished Features & Refactoring

* **GPU Hardware Acceleration / Passthrough**:
  * Implemented host GPU/CUDA library detection and exposure in [`src/sandbox/builder.rs`](src/sandbox/builder.rs).
  * Created [`web/nix-gpu-setup.sh`](web/nix-gpu-setup.sh) to handle the discovery of NVIDIA/CUDA libraries on the host and populate symlinks in a clean path `/var/run/nix-nvidia-driver/lib` without shell escaping conflicts.
  * Configured `nix-helper` to read-only bind-mount host library paths (`/usr/lib64`, `/lib64`, `/usr/lib`, `/lib`) and the symlink directory inside the chroot jail when storage sandboxing is enabled.
  * Added environment variables (`LD_LIBRARY_PATH=/run/opengl-driver/lib`, `LIBVA_DRIVERS_PATH=/usr/lib64/dri`) to allow Nix-built packages (such as `ffmpeg`) to load host drivers.
  * Added unit test cases `test_build_bwrap_command_gpu` and `test_build_bwrap_command_gpu_sandboxed` in [`src/sandbox/builder_tests.rs`](src/sandbox/builder_tests.rs) to verify output command matching.
* **Remote Deployment & Verification**:
  * Built and deployed the updated `nix-helper` release binary and assets to the Unraid test host (`100.68.35.70`).
  * Reinstalled the Jellyfin service on the Unraid test host and verified that `ffmpeg -init_hw_device cuda=cu:0 -version` successfully initializes the host's dual RTX 4060 Ti GPUs from within the chroot jail.
* **Git Sync**:
  * All code modifications have been committed and successfully pushed to `origin/main` on GitHub.

---

## 2. Next Steps / Current Action Item

1. **Test Jellyfin Transcoding playback**: Perform a live play test of an AV1/HEVC file to confirm Jellyfin transcodes to H.264 using the GPUs.
2. **Review UI/Theme Integration**: Check if the folder drop-down box styling and plugin colors blend perfectly with Unraid's theme.
