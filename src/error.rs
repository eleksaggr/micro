use core::fmt::Display;

pub trait Error: Display {
    fn description(&self) -> &str;
    fn cause(&self) -> Option<&Error> {
        None
    }
}
