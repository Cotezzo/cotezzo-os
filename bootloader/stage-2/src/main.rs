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
mod vga;    // Use VGA module
mod pmio;   // Make PMIO module visible to VGA module
mod print;
mod fs;

use fs::FS;
use print::ToString;
use vga::{get_vga, Vga};
use core::arch::asm;

/*  All the code written here is underneath the .text.rs_start section.
    The _start section is then placed above all else by the linker script. */
//core::arch::global_asm!(".section .text.rs_start");

/** A main doesn’t make sense without an underlying runtime that calls it.
    We are overwriting the os entry point with our own _start function.
    The no_mangle attribute ensures that the compiler outputs the
    function with name _start and not some cryptic unique name symbol.
    We also have to mark the function as extern "C" to tell the compiler that
    it should use the C calling convention for this function
    (https://en.wikipedia.org/wiki/X86_calling_conventions).
    
    DS, SS, ES are set to 0x10, refers to 32pm data segment selector.
    CS register is set to 0x08, refers to 32pm code segment selector.
    SP is set to 0; since it grows backwards, it will wrap around the segment.
//! RAM size for this device is 128MB (134_217_728 or 0x800_0000 byte).
//! "Wrapping around" doesn't work: 0xFFFFFFFF isn't always a valid memory address.
//! ESP is instead set to 0xFFFF. Read main.asm for more details.
    The stack will override our code if we reach it (flat mem model = 4GB...):
    in this environment there are no "stack overflow guards". */
#[no_mangle]
pub extern "C" fn _rs_start(drive_number: u32) -> ! {

    // Get VGA driver static instance, clear screen from BIOS and stage-1 text
    let vga: &mut Vga = get_vga();
    vga.clear();
    vga.clear_cursor();

    // Stage-2 started, say hi
    println!("Hello world from main.rs! Current disk: ", drive_number);

    // Initialize Fat12 "driver"
    let fat12: FS = FS::new(drive_number as u8);
    
    // Stage-2 completed, start the kernel
    println!("Starting Kernel...");

    // Do nothing until the end of time - 'never' (!) return type
    loop {}
}



/*
trait Slice { fn read(&self, start: usize, end: usize) -> &[u8]; }
impl Slice for [u8;512] {
    fn read(&self, start: usize, end: usize) -> &[u8] {
        if start > end { panic!("Buffer::read: start > end"); }
        if end > self.len() { panic!("Buffer::read: end > buffer.len()"); }

        let read_only = unsafe { core::slice::from_raw_parts(self as *const u8, 512) };
        &read_only[start..end]
    }
}
*/

/* ==== PANIC HANDLER ======================================================= */
use core::{panic::PanicInfo};

use crate::print::ToStringBase;

/** "panic_handler" defines the method that is invoked when a panic occurs.
    In a no_std environment we need to define it ourselves. */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {

    // Print panic reason
    println!("\r\nPanic: ", _info.message().unwrap().as_str().expect("Panic!"));
    
    // Do nothing until the end of time - 'never' (!) return type
    loop {}
}

/* ==== TESTS =============================================================== */
// TODO: custom framework for no_std testing