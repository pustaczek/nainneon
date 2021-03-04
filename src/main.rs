#![feature(custom_test_frameworks)]
#![test_runner(crate::test::runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]
#![no_std]

mod qemu;
#[macro_use]
mod serial;
mod test;
mod vga;

#[cfg(not(test))]
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    println!("Hello, world!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_style(vga::Color::Black, vga::Color::LightRed);
    println!("{}", info);
    vga::reset_style();
    loop {}
}
