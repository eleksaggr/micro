use core::fmt;
use core::ptr::Unique;
use volatile::Volatile;
use spin::Mutex;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    fn from_int(i: u8) -> Color {
        match i {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::White,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub fn get_fg(&self) -> Color {
        Color::from_int(self.0 & 0xF)
    }

    pub fn get_bg(&self) -> Color {
        Color::from_int((self.0 & 0xF0) >> 4)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct VGAChar {
    character: u8,
    color: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<VGAChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column: 0,
    color: ColorCode::new(Color::White, Color::Black),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) },
});

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga::print(format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub struct Writer {
    column: usize,
    color: ColorCode,
    buffer: Unique<Buffer>,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.shift(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.shift();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column;

                let color = self.color;
                self.buffer().chars[row][col].write(VGAChar {
                    character: byte,
                    color: color,
                });

                self.column += 1;
            }
        }
    }


    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    fn shift(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let buffer = self.buffer();
                let character = buffer.chars[row][col].read();
                buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = VGAChar {
            character: b' ',
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer().chars[row][col].write(blank);
        }
    }

    pub fn get_color(&self) -> ColorCode {
        self.color
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = ColorCode::new(fg, bg);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
