use x86_64::instructions::port::Port;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[allow(dead_code)]
pub fn exit(code: ExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(code as u32);
    }
}
