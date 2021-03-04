use crate::qemu;
use core::panic::PanicInfo;

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("test {} ... ", core::any::type_name::<T>());
        self();
        serial_println!("\x1B[1;32mok\x1B[0m");
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("\x1B[1;31mFAILED\x1B[0m\n\n{}\n", info);
    qemu::exit(qemu::ExitCode::Failure);
    loop {}
}

#[cfg(test)]
pub fn runner(tests: &[&dyn Testable]) {
    serial_println!("\nrunning {} tests", tests.len());
    for test in tests {
        test.run();
    }
    serial_println!("\ntest result: \x1B[1;32mok\x1B[0m. {} passed", tests.len());
    qemu::exit(qemu::ExitCode::Success);
}

#[test_case]
fn easy_assertion() {
    assert_eq!(2 + 2, 4);
}
