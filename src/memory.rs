pub unsafe fn init(
    physical_memory_offset: x86_64::VirtAddr,
) -> x86_64::structures::paging::OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    x86_64::structures::paging::OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(
    physical_memory_offset: x86_64::VirtAddr,
) -> &'static mut x86_64::structures::paging::PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut x86_64::structures::paging::PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub struct EmptyFrameAllocator;

unsafe impl x86_64::structures::paging::FrameAllocator<x86_64::structures::paging::Size4KiB>
    for EmptyFrameAllocator
{
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static bootloader::bootinfo::MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static bootloader::bootinfo::MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = x86_64::structures::paging::PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions =
            regions.filter(|r| r.region_type == bootloader::bootinfo::MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| {
            x86_64::structures::paging::PhysFrame::containing_address(x86_64::PhysAddr::new(addr))
        })
    }
}

unsafe impl x86_64::structures::paging::FrameAllocator<x86_64::structures::paging::Size4KiB>
    for BootInfoFrameAllocator
{
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
