use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

pub macro serial_print($($arg:tt)*) {
    $crate::serial::_print(format_args!($($arg)*));
}

pub macro serial_println {
    () => ($crate::serial_print!("\n")),
    ($($arg:tt)*) => ($crate::serial::serial_print!("{}\n", format_args!($($arg)*))),
}

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("writing to serial failed");
}
