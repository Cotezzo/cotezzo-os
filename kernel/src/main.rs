/* ==== FEATURES ============================================================ */
#![no_std]
#![no_main]
#![feature(panic_info_message)]

/* ==== MODULES ============================================================= */
use core::panic::PanicInfo;
use vga::get_vga;
use prints::ToString;

mod vga;    // Use VGA module
mod pmio;   // Make PMIO module visible to VGA module
mod prints;
mod hal;

/* ==== ENTRY POINT ========================================================= */
#[no_mangle] pub extern "C" fn _rs_start() -> ! {

    // Clear text and cursor from stage-2
    get_vga().clear_screen();

    // Load kernel GDT and IDT
    hal::init();

    // TODO: something...
    println!("Kernel starting..!");
    
    unsafe { core::arch::asm!( "int 0" ); }
    unsafe { core::arch::asm!( "int 1" ); }
    unsafe { core::arch::asm!( "int 255" ); }

    // Do nothing until the end of time - 'never' (!) return type
    loop {}
}

/* ==== PANIC HANDLER ======================================================= */
#[panic_handler] fn panic(_info: &PanicInfo) -> ! {

    // Print panic reason
    println!("Panic: ", _info.message().unwrap().as_str().unwrap_or("Unknown"));
    
    // Do nothing until the end of time - 'never' (!) return type
    loop {}
}

/* ==== TESTS =============================================================== */
// TODO: custom framework for no_std testing