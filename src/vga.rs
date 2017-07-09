use core::fmt;
use core::ptr::Unique;
use sync::Mutex;
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
    /// Returns the `Color` associated with the given index. These values can be found
    /// [here](wiki.osdev.org/Text_UI#Colours).
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
/// A representation of a character together with its foreground and background colors.
struct VGAChar {
    character: u8,
    color: ColorCode,
}

///
struct Buffer {
    chars: [[Volatile<VGAChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// An interface dedicated to writing to the [VGA text
/// buffer](https://en.wikipedia.org/wiki/VGA-compatible_text_mode#Text_buffer) located in memory
/// at the phyiscal memory address `0xB8000`. The `Writer` is the only owner of the memory
/// location, making it impossible to have multiple of them concurrently.
pub struct Writer {
    /// The column the `Writer` will write the next [`VGAChar`](struct.VGAChar.html) to.
    column: usize,
    /// The [`ColorCode`](struct.ColorCode.html) that specifies the colors for the next [`VGAChar`](struct.VGAChar.html).
    color: ColorCode,
    /// A `Unique` pointer to the underlying [`Buffer`](struct.Buffer.html).
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

    /// Writes a byte to the VGA buffer at the current position.
    ///
    /// The special character `\n` is interpreted as a line break,
    /// and results in a call to [`Writer::shift()`](struct.Writer.html#method.shift)
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


    /// Returns a mutable reference to the underlying buffer of the `Writer`.
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

    /// Clears the specified row, by overwriting it with SPACE(ASCII: 0x20) characters.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut w: Writer = Writer::new(Color::White, Color::Black);
    /// w.clear_row(0);
    /// ```
    fn clear_row(&mut self, row: usize) {
        let blank = VGAChar {
            character: b' ',
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer().chars[row][col].write(blank);
        }
    }

    /// Sets the colors the `Writer` will use for future writing.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut w: Writer = Writer::new(Color::White, Color::Black);
    /// w.set_color(Color::Red, Color::Green);
    /// ```
    pub fn set_colors(&mut self, foreground: Color, background: Color) {
        self.color = ColorCode::new(foreground, background);
    }

    /// Returns the currently used colors as a tuple,
    /// with the first element being the foreground and
    /// the second the background.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut w: Writer = Writer::new(Color::White, Color::Black);
    /// let colors = w.colors();
    /// assert_eq!(Color::White, colors.0);
    /// assert_eq!(Color::Black, colors.1);
    /// ```
    pub fn colors(&self) -> (Color, Color) {
        (
            Color::with(self.color.0 >> 4),
            Color::with(self.color.0 & 0xF),
        )
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
