// VGA colors

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black = 0x00,
    Blue = 0x01,
    Green = 0x02,
    Cyan = 0x03,
    Red = 0x04,
    Magenta = 0x05,
    Brown = 0x06,
    LightGray = 0x07,
    DarkGray = 0x08,
    LightBlue = 0x09,
    LightGreen = 0x0a,
    LightCyan = 0x0b,
    LightRed = 0x0c,
    LightMagenta = 0x0d,
    Yellow = 0x0e,
    White = 0x0f,
}

// VGA attribute

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Attribute(u8);

impl Default for Attribute {
    fn default() -> Self {
        Attribute((Color::Black as u8) << 4 | (Color::White as u8))
    }
}

impl Attribute {
    pub fn new(foreground: Color, background: Color) -> Self {
        Attribute((background as u8) << 4 | (foreground as u8))
    }
}

// VGA single character

#[derive(Clone, Copy)]
#[repr(C)]
struct VgaCharacter {
    character: u8,
    attribute: Attribute,
}

// VGA buffer

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
pub const CCSID_437: &str = "\
    ?☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼?!\"#$%&'()*+,-./0123456789:;<=>?\
    @ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~⌂\
    ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐\
    └┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■?\
";

#[repr(transparent)]
struct Buffer {
    inner: [[volatile::Volatile<VgaCharacter>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// VGA writer

pub struct Writer {
    row_position: usize,
    col_position: usize,
    attribute: Attribute,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn set_attribute(&mut self, fg: Color, bg: Color) {
        self.attribute = Attribute::new(fg, bg);
    }

    pub fn reset_attribute(&mut self) {
        self.attribute = Attribute::default();
    }

    fn update_cursor(&mut self) {
        let pos = (self.row_position * BUFFER_WIDTH + self.col_position) as u16;
        let mut port = x86_64::instructions::port::Port::new(0x3d4);
        unsafe {
            port.write(pos << 8 | 0x0f);
            port.write(pos & 0xff00 | 0x0e);
        }
    }

    fn clear(&mut self, row: usize, col: usize) {
        self.buffer.inner[row][col].write(VgaCharacter {
            character: ' ' as u8,
            attribute: Attribute::default(),
        });
    }

    fn write_byte(&mut self, byte: u8) {
        if self.col_position >= BUFFER_WIDTH {
            self.new_line();
        }
        self.buffer.inner[self.row_position][self.col_position].write(VgaCharacter {
            character: byte,
            attribute: self.attribute,
        });
        self.col_position += 1;
    }

    fn write_string(&mut self, string: &str) {
        string.chars().into_iter().for_each(|ch| match ch {
            '\n' | '\r' => self.new_line(),
            '\t' => self.write_string("    "),
            '\x00'..='\x7f' => self.write_byte(ch as u8),
            _ => match CCSID_437.chars().position(|c| c == ch) {
                Some(c) => self.write_byte(c as u8),
                None => self.write_byte('?' as u8),
            },
        });
    }

    fn new_line(&mut self) {
        self.row_position += 1;
        self.col_position = 0;
        if self.row_position < BUFFER_HEIGHT {
            return;
        }
        (1..BUFFER_HEIGHT).for_each(|row| {
            (0..BUFFER_WIDTH).for_each(|col| {
                let vga_char = self.buffer.inner[row][col].read();
                self.buffer.inner[row - 1][col].write(vga_char);
                if row == BUFFER_HEIGHT - 1 {
                    self.clear(row, col);
                }
            })
        });
        self.row_position -= 1;
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer {
        row_position: 0,
        col_position: 0,
        attribute: Attribute::default(),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! color {
    (($fg:expr, $bg:expr), $($print_statement:stmt),*) => {
        $crate::vga::WRITER.lock().set_attribute($fg, $bg);
        $($print_statement)*
        $crate::vga::WRITER.lock().reset_attribute();
    };
    ($fg:expr, $($print_statement:stmt),*) => {
        $crate::vga::WRITER.lock().set_attribute($fg, $crate::vga::Color::Black);
        $($print_statement)*
        $crate::vga::WRITER.lock().reset_attribute();
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
        WRITER.lock().update_cursor();
    });
}

pub fn init() {
    (0..BUFFER_HEIGHT).for_each(|row| {
        (0..BUFFER_WIDTH).for_each(|col| {
            WRITER.lock().clear(row, col);
        });
    });
}
