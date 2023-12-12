/* Code and comments from: https://os.phil-opp.com/freestanding-rust-binary/  */

/*  This is a bare metal implementation. This means that the final binary won't
    be linked to the standard library that interacts with the operating system,
    since there is not operating system (hence, bare metal).
    no_std tells the compiler not to link the standard library. */
#![no_std]

/*  Freestanding executable, no access to Rust runtime and crt0.
    We need to define our own entry point overwriting the crt0 one directly.
    no_main tells the compiler not to use the normal entry point chain. */
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

/*  panic_handler defines the method that is invoked when a panic occurs.
    In a no_std environment we need to define it ourselves. */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/*  A main doesn’t make sense without an underlying runtime that calls it.
    We are overwriting the os entry point with our own _start function.
    The no_mangle attribute ensures that the compiler outputs the
    function with name _start and not some cryptic unique name symbol.
    Required since we need to tell the entry point name to the linker.
    "_start" is the default entry point name for most systems.
    We also have to mark the function as extern "C" to tell the compiler that
    it should use the C calling convention for this function.
    (https://en.wikipedia.org/wiki/X86_calling_conventions) */
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { asm!(
        "mov ah, 0x0E",
        "mov bh, 0",
        "mov al, 'a'",
        "int 0x10"
    ) };

    loop {}
}