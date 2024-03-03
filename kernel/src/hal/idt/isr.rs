use crate::println;
use crate::prints::{ToString};
use crate::{get_vga};

use super::entry::IdtEntry;
use super::Idt;

/* ==== ASM EXTERN METHODS + MAPPING ======================================== */
// Interrupt methods declared as global in isr.asm - to be set in the IDT.
extern "C" {
    fn _c_isr_0();
    fn _c_isr_1();
    // ... TODO: macro to generate up to 255
    fn _c_isr_255();
}

/// Define and set IDT entries with gate informations and ASM ISRs pointers.
pub fn init_isrs(idt: &mut Idt) {
    idt.set(0u8, IdtEntry::new(_c_isr_0, IdtEntry::FLAG_GATE_INTERRUPT_32BIT, IdtEntry::PRIVILEGE_LEVEL_RING0)); 
    idt.set(1u8, IdtEntry::new(_c_isr_1, IdtEntry::FLAG_GATE_INTERRUPT_32BIT, IdtEntry::PRIVILEGE_LEVEL_RING0)); 
    // ... TODO: macro to generate up to 255
    idt.set(255u8, IdtEntry::new(_c_isr_255, IdtEntry::FLAG_GATE_INTERRUPT_32BIT, IdtEntry::PRIVILEGE_LEVEL_RING0));
}

/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct IsrStackFrame {
    // TODO: map CPU saved state data, pusha, ...
    // Dispatcher pushed data
    ds: u32,
    pusha_edi: u32, pusha_esi: u32, pusha_ebp: u32, pusha_esp: u32, pusha_ebx: u32, pusha_edx: u32, ecx: u32, pusha_eax: u32,

    // CPU pushed data
    interrupt: u32,
    error: u32,
    prev_eip: u32, prev_cs: u32, prev_eflags: u32,

    // ! For CS != Kernel CS only (stack/ring switch)
    prev_esp: u32, prev_ss: u32
}

/* ==== COMMON INTERRUPT DISPATCHER ========================================= */
/// Extern method esposed to the linker and called by the assembly module that
/// defines all the ISRs (isr.asm). It handles all the interrupts (0-255), even
/// if there is no actual implementation (this method would throw an error).
/// This method then dispatches the interrupt and calls the correct handler.
/// Optionally, the actual handler can be directly linked to the _c_isr_<n>.
#[no_mangle] pub extern "C" fn _rs_isr_dispatcher(data: *const IsrStackFrame) {
    let interrupt =  unsafe { (*data).interrupt };
    println!("Interrupt received: ", interrupt);
}