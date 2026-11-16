use tracing::Level;

pub fn log_init(service: &'static str) {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    tracing::info!(service, "logging ready")
}
