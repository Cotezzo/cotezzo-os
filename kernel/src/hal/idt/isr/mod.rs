mod isrs;
mod isr_0;

use crate::{get_vga, println, ToString};

/* ==== TYPE DEFINITION ===================================================== */
/// The CPU and the ASM-defined dispatcher push to the stack some informations.
/// The argument given to the Rust dispatcher is a pointer to the stack frame
/// containing these informations, mapped here.
#[repr(C, packed)]
pub struct IsrStackFrame {
    // Dispatcher pushed data
    pub ds: u32,
    pub pusha_edi: u32, pusha_esi: u32, pusha_ebp: u32, pusha_esp: u32, pusha_ebx: u32, pusha_edx: u32, ecx: u32, pusha_eax: u32,

    // CPU pushed data
    pub interrupt: u32,
    pub error: u32,
    pub prev_eip: u32, prev_cs: u32, prev_eflags: u32,

    // ! For CS != Kernel CS only (stack/ring switch)
    pub prev_esp: u32, prev_ss: u32
}

/// ISR Handler interface
pub type Isr = fn(*const IsrStackFrame) -> ();

/* ==== DISPATCHER ========================================================== */
/// Extern method esposed to the linker and called by the assembly module that
/// defines all the ISRs (isr.asm). It handles all the interrupts (0-255), even
/// if there is no actual implementation (this method would throw an error).
/// This method then dispatches the interrupt and calls the correct handler.
/// Optionally, the actual handler can be directly linked to the _c_isr_<n>.
#[no_mangle] pub extern "C" fn _rs_isr_dispatcher(data: *const IsrStackFrame) {
    let interrupt =  unsafe { (*data).interrupt } as u8;
    let error =  unsafe { (*data).error };
    println!("Interrupt received: ", interrupt);
    println!("Error received: ", error);

    // Try to retrieve handler for received interrupt
    let handler: *const Isr = unsafe { ISRS[interrupt as usize] };

    // If there is no ISR associated with this interrupt, halt execution
    if handler.is_null() {
        panic!("Unhandled interrupt!");
    }

    let handler: Isr = unsafe { core::mem::transmute(handler) };
    handler(data);
}

/* ==== ISRS DATA and INITIALIZATION ======================================== */
/// Internal mapping between interrupts and handlers.
/// If the handler pointer for the received interrupt is null, panic.
pub static mut ISRS: [*const Isr; 256] = unsafe { core::mem::zeroed() };

/// Initializes IDT with ASM method pointers and map specific handlers and
/// their associated interrupts.
pub fn init(idt: &mut super::Idt) {
    isrs::init(idt);

    // Division by 0
    unsafe { ISRS[0] = isr_0::handler as *const Isr; }
}