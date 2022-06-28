use log::{Log, Record, Metadata};

pub struct Logger;

impl Log for Logger {
    // 提供讓使用者偵測有沒有啟用 log 的 API
    fn enabled(&self, _meta: &Metadata) -> bool {
        true
    }

    // 實際輸出 log 的部份
    fn log(&self, record: &Record) {
        eprintln!("{}: {}", record.level(), record.args());
    }

    // 清空緩衝區
    fn flush(&self) {}
}