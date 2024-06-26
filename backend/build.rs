use std::path::Path;
use std::process::Command;

const FRONTEND_DIR: &str = "../frontend";

fn main() {
    println!("cargo:rerun-if-changed={}/src", FRONTEND_DIR);
    println!("cargo:rerun-if-changed={}/index.html", FRONTEND_DIR);
    build_frontend(FRONTEND_DIR);
}

/// build frontend at src
fn build_frontend<P: AsRef<Path>>(source: P) {
    // if on debug mode, build frontend in debug mode
    #[cfg(debug_assertions)]
    Command::new("trunk")
        .args(["build"]) //, "--release"
        .current_dir(source.as_ref())
        .status()
        .expect("Failed to build Frontend");
    // if on release mode, build frontend in release mode
    #[cfg(not(debug_assertions))]
    Command::new("trunk")
        .args(["build", "--release"]) //, "--release"
        .current_dir(source.as_ref())
        .status()
        .expect("Failed to build Frontend");
}
