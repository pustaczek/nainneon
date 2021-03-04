use core::fmt;
use spin::mutex::SpinMutex;
use volatile::Volatile;

pub macro print($($arg:tt)*) {
    $crate::vga::_print(format_args!($($arg)*))
}

pub macro println {
    () => ($crate::vga::print!("\n")),
    ($($arg:tt)*) => ($crate::vga::print!("{}\n", format_args!($($arg)*))),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
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

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct Style(u8);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Pixel {
    ascii: u8,
    style: Style,
}

#[repr(transparent)]
struct Buffer {
    grid: [[Pixel; WIDTH]; HEIGHT],
}

struct Terminal {
    column: usize,
    row: usize,
    style: Style,
}

const WIDTH: usize = 80;
const HEIGHT: usize = 25;

static TERMINAL: SpinMutex<Terminal> = SpinMutex::new(Terminal {
    column: 0,
    row: 0,
    style: Style::new(Color::White, Color::Black),
});

impl Style {
    const fn new(foreground: Color, background: Color) -> Style {
        Style(((background as u8) << 4) | (foreground as u8))
    }
}

impl Terminal {
    pub fn write(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                b'\n' => self.write_newline(),
                0x20..=0x7e => self.write_raw_byte(byte),
                _ => self.write_raw_byte(0xfe),
            }
        }
    }

    fn write_newline(&mut self) {
        if self.row == HEIGHT - 1 {
            self.shift_up();
            self.clear_row(self.row);
            self.column = 0;
        } else {
            self.row += 1;
            self.column = 0;
        }
    }

    fn write_raw_byte(&mut self, ascii: u8) {
        if self.column >= WIDTH {
            self.write_newline();
        }
        let pixel = Pixel {
            ascii,
            style: self.style,
        };
        set_pixel(self.row, self.column, pixel);
        self.column += 1;
    }

    fn shift_up(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                set_pixel(row - 1, col, get_pixel(row, col));
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Pixel {
            ascii: b' ',
            style: self.style,
        };
        for col in 0..WIDTH {
            set_pixel(row, col, blank);
        }
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| TERMINAL.lock().write_fmt(args))
        .unwrap();
}

#[allow(dead_code)]
pub fn set_style(foreground: Color, background: Color) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        TERMINAL.lock().style = Style::new(foreground, background)
    });
}

#[allow(dead_code)]
pub fn reset_style() {
    set_style(Color::White, Color::Black);
}

fn get_pixel(row: usize, col: usize) -> Pixel {
    volatile_buffer()
        .map(|buffer| &buffer.grid[row][col])
        .read()
}

fn set_pixel(row: usize, col: usize, pixel: Pixel) {
    volatile_buffer()
        .map_mut(|buffer| &mut buffer.grid[row][col])
        .write(pixel)
}

fn volatile_buffer() -> Volatile<&'static mut Buffer> {
    Volatile::new(unsafe { &mut *(0xb8000 as *mut Buffer) })
}
