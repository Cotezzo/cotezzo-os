/* Inspired by: https://os.phil-opp.com/freestanding-rust-binary/  */

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
/*  Import "lib_to_string.rs" methods and types (even if not explicitly
    used: print_string is a macro and uses them in this scope).
    TODO: uncomment once pointers are fixed

mod lib_print;

use crate::lib_print::print_string;
use crate::lib_print::ToString;

mod lib_disk;
use crate::lib_disk::_disk_reset;
use crate::lib_disk::_disk_get_params;
*/

/*  All the code written here is underneath the .text._start section.
    The _start section is then placed above all else by the linker script. */
core::arch::global_asm!(".section .text._start");

/*  A main doesn’t make sense without an underlying runtime that calls it.
    We are overwriting the os entry point with our own _start function.
    The no_mangle attribute ensures that the compiler outputs the
    function with name _start and not some cryptic unique name symbol.
    Required since we need to tell the entry point name to the linker.
    "_start" is the default entry point name for most systems.
    We also have to mark the function as extern "C" to tell the compiler that
    it should use the C calling convention for this function
    (https://en.wikipedia.org/wiki/X86_calling_conventions).
    
    DS, SS, CS, ES registries are set to 0x1000, SP is set to 0 by stage-1.
    Since the Stack Pointer grows backwards, it will wrap around the segment.
    The stack will still override our code if we reach 64kB.
    In this environment there are no "stack overflow guards". */
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // To print character
    let char: u8 = b'0';

    // First method to be called correctly prints '0' (original variable value);
    // Second method DOES NOT, prints random character;
    // This is true for both combinations (value_print first, pointer_print first).
    _value_print_test(char, 0);
    _pointer_print_test(&char as *const u8);

    // Do nothing until the end of time
    loop {}
}

// TODO: remove once pointers are fixed
extern "C" {
    fn _pointer_print_test(char: *const u8) -> ();
    fn _value_print_test(char: u8, page: u8) -> ();
}

/* ==== PANIC HANDLER ======================================================= */
use core::panic::PanicInfo;

/*  panic_handler defines the method that is invoked when a panic occurs.
    In a no_std environment we need to define it ourselves. */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {

    /* Print panic reason
    TODO: uncomment once pointers are fixed
    print!(_info.message().unwrap().as_str().expect("Panic!"));
    */

    // Do nothing until the end of time
    loop {}
}