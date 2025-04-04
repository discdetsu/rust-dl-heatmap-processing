use log::{info, LevelFilter, Record};
use env_logger::{Builder, fmt::Formatter};
use std::cell::RefCell;
use std::io::Write;

// Thread-local storage for request_id
thread_local! {
    static REQUEST_ID: RefCell<Option<String>> = RefCell::new(None);
}

// Function to set request_id for this thread
pub fn set_request_id(id: &str) {
    REQUEST_ID.with(|req_id| *req_id.borrow_mut() = Some(id.to_string()));
}

pub fn setup_logger() {
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Debug);
    builder.format(|buf: &mut Formatter, record: &Record| {
        let request_id = REQUEST_ID.with(|req_id| req_id.borrow().clone().unwrap_or_else(|| "".to_string()));
        
        writeln!(
            buf,
            "Request id : {:<6} | {:<8} | {}:{} | {}",
            request_id,
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    });
    builder.init();
}
