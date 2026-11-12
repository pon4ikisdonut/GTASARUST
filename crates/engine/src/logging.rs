use tracing_subscriber::EnvFilter;

/// Initialize structured `tracing` logging for GTASARUST tools and game binary.
/// Uses `RUST_LOG` if set, otherwise defaults to `info` for workspace crates.
pub fn init_logging(verbose: bool) {
    let default_filter = if verbose {
        "debug,wgpu_core=warn,wgpu_hal=warn,naga=warn"
    } else {
        "info,wgpu_core=warn,wgpu_hal=warn,naga=warn"
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(false)
        .try_init();
}
