#!/bin/bash
# unraid-nix GPU/CUDA/VA-API Driver Passthrough Setup Script
# This script prepares the symlink directory for NVIDIA/CUDA libraries on the host.

set -e

# Create target directory
mkdir -p /var/run/nix-nvidia-driver/lib
rm -rf /var/run/nix-nvidia-driver/lib/*

# Symlink all NVIDIA/CUDA libraries
if [ -d /usr/lib64 ]; then
    for f in /usr/lib64/libcuda.so* /usr/lib64/libnvidia-*.so*; do
        if [ -f "$f" ] || [ -L "$f" ]; then
            ln -sf "$f" /var/run/nix-nvidia-driver/lib/$(basename "$f")
        fi
    done
fi
