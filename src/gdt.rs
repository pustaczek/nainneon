use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
}

pub const DOUBLE_FAULT_IST: usize = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST] = {
            const STACK_SIZE: usize = 4096 * 8;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code, tss })
    };
}

pub fn init() {
    GDT.0.load();
    unsafe {
        x86_64::instructions::segmentation::set_cs(GDT.1.code);
        x86_64::instructions::tables::load_tss(GDT.1.tss);
    }
}
