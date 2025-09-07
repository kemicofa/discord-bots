pub fn init_tracing() {
    // This sets up a global subscriber based on RUST_LOG (defaults to info/warn)
    tracing_subscriber::fmt().init();
}