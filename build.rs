use std::env;

fn main() {
    // Only generate man pages in release builds or when explicitly requested
    if env::var("PROFILE").unwrap_or_default() != "release" && env::var("GENERATE_MAN").is_err() {
        return;
    }

    // We can't easily import the CLI struct here, so we'll generate a basic man page
    // The full man page generation happens at runtime via a hidden command
    println!("cargo:rerun-if-changed=src/cli/mod.rs");
}
