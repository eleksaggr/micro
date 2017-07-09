use core::fmt;
use sync::Mutex;
use vga::{Color, WRITER};

macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => ({
        let level = $lvl;
        $crate::util::log::LOGGER.lock().log(level, format_args!($($arg)+));
    })
}

lazy_static! {
    pub static ref LOGGER: Mutex<PrintLogger> = Mutex::new(PrintLogger::new(Level::Info));
}

pub trait Logger {
    fn log(&mut self, level: Level, args: fmt::Arguments);
}

pub struct PrintLogger {
    level: Level,
}

impl PrintLogger {
    pub fn new(level: Level) -> PrintLogger {
        PrintLogger { level: level }
    }

    pub fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl Logger for PrintLogger {
    fn log(&mut self, level: Level, args: fmt::Arguments) {
        if level >= self.level {
            let colors = {
                let mut writer = WRITER.lock();
                match level {
                    Level::Info => writer.set_colors(Color::White, Color::Black),
                    Level::Warn => writer.set_colors(Color::Yellow, Color::Black),
                    Level::Error => writer.set_colors(Color::Red, Color::Black),
                }
                writer.colors()
            };

            println!("[{}] {}", level, args);
            WRITER.lock().set_colors(colors.0, colors.1);
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
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
