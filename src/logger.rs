use log::LevelFilter;
use env_logger::{Builder, fmt::Formatter};
use std::io::Write;


pub fn setup_logger() {
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Debug);
    builder.format(|buf: &mut Formatter, record: &log::Record| {
        writeln!(
            buf,
            "Request id : {:<6} | {:<8} | {}:{} | {}",
            "", // Placeholder for request_id
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    });
    builder.init();
}
