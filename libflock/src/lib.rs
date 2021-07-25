// TODO: Automate the generation of the C header file
// whenever the crate is published.
#![no_std]

pub mod signals;

#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo) -> ! {
    loop {}
}
