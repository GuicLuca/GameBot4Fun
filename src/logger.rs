use std::env;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()|{
            let level_filter: LevelFilter = match env::var("LOG_LEVEL").expect("Expected an env file with the LOG_LEVEL entry set. {}").as_str() {
                "Off" => LevelFilter::Off,
                "Trace" => LevelFilter::Trace,
                "Debug" => LevelFilter::Debug,
                "Info" => LevelFilter::Info,
                "Warn" => LevelFilter::Warn,
                "Error" => LevelFilter::Error,
                _ => LevelFilter::Info,
            };
            log::set_max_level(level_filter)
        })
}

static LOGGER: SimpleLogger = SimpleLogger;