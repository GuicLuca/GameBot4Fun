use std::env;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

/**
 * This is the logger configuration of the bot.
 * => In the whole project use log marco (info!, error!, ...) 
 * and the logger will print every log above the log level.
 * see .env.example for log level.
 */
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

/**
 * This method set the logger an initialize the loglevel.
 * It MUST be called in the beginning of the program !
 *
 * @return Result<(), SetLoggerError>
 */
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