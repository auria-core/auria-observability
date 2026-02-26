use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

pub fn init_logging(level: &str) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level.parse().unwrap_or(Level::INFO))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

pub fn log_info(message: &str) {
    info!("{}", message);
}

pub fn log_warn(message: &str) {
    warn!("{}", message);
}

pub fn log_error(message: &str) {
    error!("{}", message);
}
