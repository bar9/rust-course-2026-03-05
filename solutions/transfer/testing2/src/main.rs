#![no_std]
#![no_main]

use testing2::DataBuffer;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// Minimal embedded entry point
#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut buffer = DataBuffer::<5>::new();

    buffer.push(1);
    buffer.push(2);
    buffer.push(3);

    let _sum = buffer.sum();

    loop {}
}