#!/bin/bash
# Prototype for Unraid Nix Storage Sandbox
# This script runs inside a private mount namespace to isolate /mnt access.

# 1. Self-execute inside a private mount namespace if not already running in one
if [ "$1" != "--in-namespace" ]; then
    echo "Starting prototype sandbox wrapper..."
    unshare -m "$0" --in-namespace
    exit $?
fi

echo "=== Inside Private Mount Namespace ==="

# 2. decouple mount propagation from the host namespace
mount --make-rprivate /

# 3. Create a temporary path to save access to the host's real /mnt
REAL_MNT_TEMP="/run/nix-real-mnt-$$"
mkdir -p "$REAL_MNT_TEMP"

# Bind-mount the real /mnt to the temporary path
if ! mount --bind /mnt "$REAL_MNT_TEMP"; then
    echo "ERROR: Failed to bind-mount real /mnt"
    rmdir "$REAL_MNT_TEMP"
    exit 1
fi

# 4. Overwrite /mnt with an empty RAM-based filesystem (tmpfs)
if ! mount -t tmpfs tmpfs /mnt; then
    echo "ERROR: Failed to mount tmpfs over /mnt"
    umount "$REAL_MNT_TEMP"
    rmdir "$REAL_MNT_TEMP"
    exit 1
fi

echo "Hidden real /mnt. Exposing only mock/allowed paths..."

# 5. Simulate exposing only '/mnt/user/appdata' (we will bind-mount it if it exists on host)
if [ -d "$REAL_MNT_TEMP/user/appdata" ]; then
    echo "-> Exposing /mnt/user/appdata"
    mkdir -p /mnt/user/appdata
    mount --bind "$REAL_MNT_TEMP/user/appdata" /mnt/user/appdata
fi

# 6. Simulate exposing a remote share/mount if one exists (e.g. under /mnt/remotes/ or /mnt/disks/)
# Let's find any existing unassigned devices or remotes to test binding
for subdir in remotes disks; do
    if [ -d "$REAL_MNT_TEMP/$subdir" ]; then
        echo "-> Exposing /mnt/$subdir"
        mkdir -p "/mnt/$subdir"
        mount --bind "$REAL_MNT_TEMP/$subdir" "/mnt/$subdir"
    fi
done

# 7. Unmount the real /mnt reference to complete isolation
umount -l "$REAL_MNT_TEMP"
rmdir "$REAL_MNT_TEMP"

echo "=== Sandbox Mount Isolation Complete ==="
echo ""
echo "Files visible inside /mnt in this sandbox:"
ls -la /mnt
echo ""
echo "Attempting to list /mnt/user (should only show appdata if it exists):"
ls -la /mnt/user 2>/dev/null

echo ""
echo "Prototype verification complete. Exiting namespace..."
