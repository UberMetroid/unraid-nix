#!/bin/bash
# Reloads the process-compose supervisor with the current nix.plg config.
# Invoked by nix-helper to keep the call argv-style (no shell injection surface).
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml
