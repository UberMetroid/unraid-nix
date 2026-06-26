# /etc/profile.d/nix.sh
# Adds Nix environments and profiles to the system PATH for SSH and console sessions.

if [ -e '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' ]; then
    source '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh'
fi
