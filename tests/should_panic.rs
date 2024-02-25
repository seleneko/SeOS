#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    rustos::serial_println!("[test did not panic]");
    rustos::exit_qemu(rustos::QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    rustos::serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    rustos::serial_println!("[ok]");
    rustos::exit_qemu(rustos::QemuExitCode::Success);
    loop {}
}
