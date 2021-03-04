#![no_main]
#![no_std]

mod vga;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, world!");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_style(vga::Color::Black, vga::Color::LightRed);
    println!("{}", info);
    vga::reset_style();
    loop {}
}
