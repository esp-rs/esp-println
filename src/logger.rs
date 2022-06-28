pub fn init_logger(level: log::LevelFilter) {
    unsafe {
        log::set_logger_racy(&EspLogger).unwrap();
        log::set_max_level(level);
    }
}

struct EspLogger;

impl log::Log for EspLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
