fn main() {
    // Ensure the `sqlx::migrate!` macro in this crate is recompiled whenever
    // the main migrations change, so binaries that depend on this crate
    // pick up new SQL files.
    println!("cargo:rerun-if-changed=../../migrations");
}
