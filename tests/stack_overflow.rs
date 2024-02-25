#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rustos::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rustos::serial_print!("stack_overflow::stack_overflow...\t");

    rustos::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

use lazy_static::lazy_static;
lazy_static! {
    static ref TEST_IDT: x86_64::structures::idt::InterruptDescriptorTable = {
        let mut idt = x86_64::structures::idt::InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rustos::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: x86_64::structures::idt::InterruptStackFrame,
    _error_code: u64,
) -> ! {
    rustos::serial_println!("[ok]");
    rustos::exit_qemu(rustos::QemuExitCode::Success);
    loop {}
}
