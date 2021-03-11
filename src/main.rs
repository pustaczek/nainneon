#![feature(
    abi_x86_interrupt,
    alloc_error_handler,
    const_mut_refs,
    const_raw_ptr_to_usize_cast,
    custom_test_frameworks,
    decl_macro
)]
#![test_runner(crate::test::runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]
#![no_std]

extern crate alloc;

mod allocator;
mod gdt;
mod interrupt;
mod memory;
mod qemu;
mod serial;
#[cfg(test)]
mod test;
mod vga;

use crate::vga::println;
use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader::BootInfo;
use core::alloc::Layout;
#[cfg(not(test))]
use core::panic::PanicInfo;
use x86_64::structures::paging::Page;
use x86_64::VirtAddr;

bootloader::entry_point!(main);

fn main(boot: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    gdt::init();
    interrupt::init();
    interrupt::enable();

    let physical_memory_offset = VirtAddr::new(boot.physical_memory_offset);
    let mut page_table = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { memory::BumpFrameAllocator::init(&boot.memory_map) };
    allocator::init_heap(&mut page_table, &mut frame_allocator).expect("heap init failed");

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

#[alloc_error_handler]
fn on_bad_alloc(layout: Layout) -> ! {
    panic!("allocation error with {:?}", layout);
}
