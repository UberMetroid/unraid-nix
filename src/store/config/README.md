# store::config

Generation, validation, and logging for the persistent Nix configuration.
`mod.rs` owns `log_event` (the plugin's rotating log writer with a lockfile
for concurrent processes); `generate.rs` builds the body of `nix.conf`
from the user's settings; `read.rs` provides `validate_store_path` and
`read_cfg_val` for safe lookups; `tests.rs` holds the inline tests. The
INI format read here is the same one `unraid::parse_ini_file` produces.
