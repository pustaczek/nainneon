use crate::gdt::DOUBLE_FAULT_IST;
use crate::vga;
use crate::vga::{print, println, Color};
use lazy_static::lazy_static;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Index {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(on_breakpoint);
        unsafe {
            idt.double_fault
                .set_handler_fn(on_double_fault)
                .set_stack_index(DOUBLE_FAULT_IST as u16);
        }
        idt.page_fault.set_handler_fn(on_page_fault);
        idt[Index::Timer.as_usize()].set_handler_fn(on_timer);
        idt[Index::Keyboard.as_usize()].set_handler_fn(on_keyboard);
        idt
    };
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

impl Index {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

pub fn init() {
    IDT.load();
    unsafe {
        PICS.lock().initialize();
    }
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn on_breakpoint(frame: &mut InterruptStackFrame) {
    vga::set_style(Color::Yellow, Color::Black);
    println!("x86 exception, breakpoint\n{:#?}", frame);
    vga::reset_style();
}

extern "x86-interrupt" fn on_double_fault(frame: &mut InterruptStackFrame, code: u64) -> ! {
    panic!("x86 exception, double fault {}\n{:#?}", code, frame);
}

extern "x86-interrupt" fn on_timer(_: &mut InterruptStackFrame) {
    // print!(".");
    unsafe { PICS.lock().notify_end_of_interrupt(Index::Timer.as_u8()) }
}

extern "x86-interrupt" fn on_keyboard(_: &mut InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: spin::Mutex<Keyboard<Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(chr) => print!("{}", chr),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(Index::Keyboard.as_u8()) }
}

extern "x86-interrupt" fn on_page_fault(frame: &mut InterruptStackFrame, code: PageFaultErrorCode) {
    vga::set_style(Color::Yellow, Color::Black);
    println!(
        "x86 exception, page fault {:?} at {:?}\n{:#?}",
        code,
        Cr2::read(),
        frame
    );
    vga::reset_style();
    loop {
        x86_64::instructions::hlt();
    }
}
