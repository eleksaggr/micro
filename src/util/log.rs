use core::fmt;
use spin::Mutex;

macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => ({
        let level = $lvl;
        util::log::LOGGER.lock().log(level, format_args!($($arg)+));
    })
}

lazy_static! {
    pub static ref LOGGER: Mutex<PrintLogger> = Mutex::new(PrintLogger::new(Level::Info));
}

pub trait Logger {
    fn log(&mut self, level: Level, fmt: &mut fmt::Formatter);
}

pub struct PrintLogger {
    level: Level,
}

impl PrintLogger {
    pub fn new(level: Level) -> PrintLogger {
        PrintLogger { level: level }
    }
}

impl Logger for PrintLogger {
    fn log(&mut self, level: Level, fmt: &mut fmt::Formatter) {
        println!("[{}] {}", level, fmt);
    }
}

pub enum Level {
    Info,
    Warn,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Level::Info => write!(f, "INFO"),
            &Level::Warn => write!(f, "WARN"),
            &Level::Error => write!(f, "ERR"),
        }
    }
}
