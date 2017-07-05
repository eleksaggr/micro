use core::fmt;
use spin::Mutex;

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
            // let backup = vga::WRITER.lock().get_color();
            // match level {
            //     Level::Info => {
            //         vga::WRITER
            //             .lock()
            //             .set_color(vga::Color::White, vga::Color::Black)
            //     }
            //     Level::Warn => {
            //         vga::WRITER
            //             .lock()
            //             .set_color(vga::Color::Yellow, vga::Color::Black)
            //     }
            //     Level::Error => {
            //         vga::WRITER
            //             .lock()
            //             .set_color(vga::Color::Red, vga::Color::Black)
            //     }
            // }

            println!("[{}] {}", level, args);
            // vga::WRITER
            //     .lock()
            //     .set_color(backup.get_fg(), backup.get_bg());

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
