#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = x86_64::instructions::port::Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn rustos_start() {
    rustos::color!(
        rustos::vga::Color::LightCyan,
        rustos::print!("┌{:─>78}┐", ""),
        rustos::print!(
            "│ {a:>10} rustos {v}, A simple OS kernel implemented in Rust. {a:>11} │",
            a = "",
            v = "1.0.0",
        ),
        rustos::print!("└{:─>78}┘", "")
    );
    rustos::println!();
}

pub fn vga_ccsid_437() {
    (0..rustos::vga::CCSID_437.chars().count()).for_each(|i| {
        if i % 0x40 == 0 {
            if i != 0 {
                rustos::println!();
            }
            rustos::print!("{:#04x}-{:#04x}: ", i, i + 0x3f);
        }
        rustos::print!("{}", rustos::vga::CCSID_437.chars().nth(i).unwrap());
    });
    rustos::println!();
}

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    rustos::init();
    rustos_start();
    vga_ccsid_437();

    rustos::println!("Hello World{}", "!");
    rustos::init();

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { rustos::memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { rustos::memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    rustos::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    let heap_value = alloc::boxed::Box::new(41);
    rustos::println!("heap_value at {:p}", heap_value);

    let mut vec = alloc::vec::Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    rustos::println!("vec at {:p}", vec.as_slice());

    let reference_counted = alloc::rc::Rc::new(alloc::vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    rustos::println!(
        "current reference count is {}",
        alloc::rc::Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    rustos::println!(
        "reference count is {} now",
        alloc::rc::Rc::strong_count(&cloned_reference)
    );

    #[cfg(test)]
    test_main();

    rustos::println!("OK");
    rustos::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rustos::println!("{}", info);
    rustos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rustos::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
