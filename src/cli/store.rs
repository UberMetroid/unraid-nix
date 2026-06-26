use crate::store;

pub fn setup_store(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Missing persistent path.");
        std::process::exit(1);
    }
    let path = &args[2];
    if let Err(e) = store::mount_nix_store(path) {
        eprintln!("Error mounting nix store: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = store::create_builder_accounts() {
        eprintln!("Error creating build accounts: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = store::setup_nix_conf() {
        eprintln!("Error setting up config: {}", e);
        std::process::exit(1);
    }
    println!("Nix Store successfully configured and mounted.");
}

pub fn teardown_store() {
    if let Err(e) = store::unmount_nix_store() {
        eprintln!("Error unmounting store: {}", e);
        std::process::exit(1);
    }
    println!("Nix Store successfully unmounted.");
}
