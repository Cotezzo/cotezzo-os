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

/*  Add the message() method to the PanicInfo struct in order to retrieve the
    reason and print it to the screen when the panic_handler is triggered. */
#![feature(panic_info_message)]

/* ==== ENTRY POINT ========================================================= */
/*  All the code written here is underneath the .text._start section.
    The _start section is then placed above all else by the linker script. */
    core::arch::global_asm!(".section .text._start");

    mod lib_to_string;
    use crate::lib_to_string::print_string;
    use crate::lib_to_string::ToString;
    
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
    // DS, SS, SS registries are set to 0x1000, SP is set to 0 by the stage-1.
    // Since the Stack Pointer grows backwards, it will wrap around the segment.
    // The stack will still override our code if we reach 64kB.
    //! In this environment there are no "stack overflow" guards
    
    // Only works in --release...
    print!("u8: ", 8u8, " u16: ", 16u16, " u32: ", 32u32, " u64: ", 64u64);

    // Do nothing until the end of time
    loop {}
}

/* ==== PANIC HANDLER ======================================================= */
use core::panic::PanicInfo;

/*  panic_handler defines the method that is invoked when a panic occurs.
    In a no_std environment we need to define it ourselves. */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {

    // Print panic reason
    //print_s(_info.message().unwrap().as_str().expect("Panic!"));
    print!("Panic!");

    // Do nothing until the end of time
    loop {}
}