mod logger;

use log::{info, debug, warn};

fn main() {
    logger::setup_logger();
    info!("---- Finish setting up logger ----");
    debug!("---- This is debug ----");
    warn!("---- This is warning ----");
}
