# Customization Rules for unraid-nix

## Respect Unraid System Conventions & Capabilities
- The plugin and all of its background Nix services must strictly respect Unraid's storage architecture, user shares, networking, permissions, and system options.
- At no point should any component of this plugin bypass access controls, privilege dropping, or capabilities that Unraid does not normally hand out to non-root processes (e.g., always dropping privileges to `nobody:users` using `setpriv` with configured PUID and PGID for standard flake execution).
