#![allow(dead_code)]
pub mod simple_logger {
    use log;

    use time::{format_description::FormatItem, OffsetDateTime, UtcOffset};

    use log::{Level, Metadata, Record};

    const TIMESTAMP_FORMAT_OFFSET: &[FormatItem] = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory]:[offset_minute]"
    );

    pub struct SimpleLogger {
        level: log::Level,
    }

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= self.level
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let now = format!(
                    "{}",
                    OffsetDateTime::now_utc()
                        .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap())
                        .format(TIMESTAMP_FORMAT_OFFSET)
                        .unwrap()
                );
                println!(
                    "{} - {} - [{}] {}",
                    now,
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
        }

        fn flush(&self) {}
    }

    impl Default for SimpleLogger {
        /// See [this](struct.SimpleLogger.html#method.new)
        fn default() -> Self {
            SimpleLogger::new()
        }
    }

    impl SimpleLogger {
        pub fn new() -> SimpleLogger {
            SimpleLogger { level: Level::Info }
        }

        pub fn new_with_level(level: log::Level) -> SimpleLogger {
            SimpleLogger { level }
        }

        pub fn set_level(&mut self, level: log::Level) {
            self.level = level;
        }

        pub fn init(self) -> Result<(), log::SetLoggerError> {
            log::set_max_level(self.level.to_level_filter());
            log::set_boxed_logger(Box::new(self))?;
            Ok(())
        }
    }
}

pub use simple_logger::SimpleLogger;