# sandbox::builder

Assembles the two supported isolation strategies for running a Nix service
on Unraid. `mod.rs` re-exports `build_chroot_command` and
`build_setpriv_command` and contains shared helpers like `find_nix_bash`;
`chroot.rs` and `chroot_mode.rs` build the chroot-based command
(`/nix` -> `appdata` bind, optional GPU devices); `setpriv.rs` and
`setpriv_mode.rs` build the `unshare`/`setpriv` namespace-isolation
variant. The two paths share `SandboxConfig` from the parent module.
