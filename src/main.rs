#![feature(
    abi_x86_interrupt,
    const_mut_refs,
    const_raw_ptr_to_usize_cast,
    custom_test_frameworks,
    decl_macro
)]
#![test_runner(crate::test::runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]
#![no_std]

mod gdt;
mod interrupt;
mod qemu;
mod serial;
#[cfg(test)]
mod test;
mod vga;

use crate::vga::println;
#[cfg(not(test))]
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    gdt::init();
    interrupt::init();
    interrupt::enable();

    println!("Hello, world!");

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_style(vga::Color::Black, vga::Color::LightRed);
    println!("{}", info);
    vga::reset_style();
    loop {}
}
