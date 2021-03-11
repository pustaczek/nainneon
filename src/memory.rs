use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub struct BumpFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BumpFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> BumpFrameAllocator {
        BumpFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|reg| reg.region_type == MemoryRegionType::Usable)
            .map(|reg| reg.range.start_addr()..reg.range.end_addr())
            .flat_map(|range| range.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_page_table_4(physical_memory_offset);
    OffsetPageTable::new(l4_table, physical_memory_offset)
}

unsafe fn active_page_table_4(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let frame = Cr3::read().0;
    let virt = physical_memory_offset + frame.start_address().as_u64();
    &mut *virt.as_mut_ptr()
}

pub fn create_example_page(
    page: Page,
    table: &mut OffsetPageTable,
    allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe { table.map_to(page, frame, flags, allocator) }
        .expect("map_to failed")
        .flush();
}
