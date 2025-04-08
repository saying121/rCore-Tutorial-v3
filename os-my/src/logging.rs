use log::{Level, LevelFilter, Log};

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Logger;

static LOG: Logger = Logger;

impl Logger {
    pub fn init() {
        log::set_logger(&LOG).unwrap();
        log::set_max_level(match option_env!("LOG") {
            Some(l) if l.eq_ignore_ascii_case("error") => LevelFilter::Error,
            Some(l) if l.eq_ignore_ascii_case("warn") => LevelFilter::Warn,
            Some(l) if l.eq_ignore_ascii_case("info") => LevelFilter::Info,
            Some(l) if l.eq_ignore_ascii_case("debug") => LevelFilter::Debug,
            Some(l) if l.eq_ignore_ascii_case("trace") => LevelFilter::Trace,
            _ => LevelFilter::Info,
        });
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31,
            Level::Warn => 93,
            Level::Info => 34,
            Level::Debug => 32,
            Level::Trace => 90,
        };

        println!(
            "\x1b[{}m[{:>5}]{}\x1b[0m",
            color,
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}
