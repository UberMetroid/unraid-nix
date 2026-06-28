#!/bin/bash
# Starts the nix-daemon via the standard rc script.
# Invoked by nix-helper to keep the call argv-style (no shell injection surface).
exec /etc/rc.d/rc.nix-daemon start
