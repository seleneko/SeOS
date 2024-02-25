pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub fn init() {
    use x86_64::instructions::segmentation::Segment;
    GDT.0.load();
    unsafe {
        x86_64::instructions::segmentation::CS::set_reg(GDT.1.code_selector);
        x86_64::instructions::tables::load_tss(GDT.1.tss_selector);
    }
}

use lazy_static::lazy_static;
lazy_static! {
    static ref TSS: x86_64::structures::tss::TaskStateSegment = {
        let mut tss = x86_64::structures::tss::TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = x86_64::VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (x86_64::structures::gdt::GlobalDescriptorTable, Selectors) = {
        let mut gdt = x86_64::structures::gdt::GlobalDescriptorTable::new();
        let code_selector =
            gdt.add_entry(x86_64::structures::gdt::Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(x86_64::structures::gdt::Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: x86_64::structures::gdt::SegmentSelector,
    tss_selector: x86_64::structures::gdt::SegmentSelector,
}
