mod logger;
mod config;

use log::{info, debug, warn};

fn main() {
    logger::setup_logger();
    logger::set_request_id("123456");

    let url = "localhost";

    info!("---- Finish setting up logger ----");
    info!(
        "Tuberculosis URL: {}", url
    );
}
