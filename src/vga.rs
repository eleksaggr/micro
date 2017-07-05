use core::fmt;
use core::ptr::Unique;
use spin::{Mutex, Once};
use volatile::Volatile;

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga::print(format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(Color::White, Color::Black));
}


/// The width of the VGA buffer in characters.
const BUFFER_WIDTH: usize = 80;

/// The height of the VGA buffer in characters.
const BUFFER_HEIGHT: usize = 25;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
/// The `Color` type. Holds the corresponding VGA color identifier
/// for each possible color.
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
    /// Returns the `Color` associated with the given index.
    fn with(index: u8) -> Color {
        match index {
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

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    /// Construct a new `ColorCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vga::{Color, ColorCode};
    ///
    /// let c = ColorCode::new(Color::Red, Color::Black);
    /// ```
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct VGAChar {
    character: u8,
    color: ColorCode,
}

struct Buffer {
    chars: [[Volatile<VGAChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column: usize,
    color: ColorCode,
    buffer: Unique<Buffer>,
}

impl Writer {
    /// Constructs a new `Writer` with the given color options.
    ///
    /// There may be only one `Writer` at a time, else the integrity
    /// of the VGA buffer is not given.
    ///
    /// # Examples
    ///
    /// ```
    /// use vga::Writer;
    ///
    /// let w: Writer = Writer::new(Color::White, Color::Black);
    /// ```
    pub fn new(foreground: Color, background: Color) -> Writer {
        Writer {
            column: 0,
            color: ColorCode::new(foreground, background),
            buffer: unsafe { Unique::new(0xb8000 as *mut _) },
        }
    }

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


    /// Returns the underlying buffer of the `Writer`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut w: Writer = Writer::new(Color::White, Color::Black);
    /// let buf = w.buffer();
    /// ```
    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    /// Shifts the screen up by one row.
    ///
    /// Text that is shifted out of bounds is lost.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut w: Writer = Writer::new(Color::White, Color::Black);
    /// w.shift();
    /// ```
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
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
