pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

pub fn init_heap(
    mapper: &mut impl x86_64::structures::paging::Mapper<x86_64::structures::paging::Size4KiB>,
    frame_allocator: &mut impl x86_64::structures::paging::FrameAllocator<
        x86_64::structures::paging::Size4KiB,
    >,
) -> Result<(), x86_64::structures::paging::mapper::MapToError<x86_64::structures::paging::Size4KiB>>
{
    let page_range = {
        let heap_start = x86_64::VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = x86_64::structures::paging::Page::containing_address(heap_start);
        let heap_end_page = x86_64::structures::paging::Page::containing_address(heap_end);
        x86_64::structures::paging::Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(x86_64::structures::paging::mapper::MapToError::FrameAllocationFailed)?;
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT
            | x86_64::structures::paging::PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Dummy;

unsafe impl alloc::alloc::GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: alloc::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: alloc::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}
