// Types of interrupts:
// - Exceptions: CPU errors during instructions execution
// - Hardware Interupts (or Interrupt Request, IRQ): handled by CPU die or
//   Programmable Interrupt Controller (PIC), passed to CPU one by one.
//   Can be enabled/disabled with the CLI, STI instructions.
// - Software Interrupts: INT instruction, used to implement syscalls.
//   Interrupt handlers are implemented by BIOS, Kernel...
//
// Handling an interrupt:
// - CPU receives the INT instruction with the interrupt number.
// - CPU stops executing and saves current state before handling the interrupt.
// - CPU looks up the IVT (Interrupt Vector Table, used in x86 16rm, and other
//   archs, like ARM - it's stored at address 0x0) or IDT (Interrupt Descriptor
//   Table, used in x86 32/64pm). These tables associate INT numbers (0-255) and
//   their handlers as function pointers (ISR, Interrupt Service Routine).
// - The handler is executed, and the last instructin must be IRET.
// - IRET restores the previously saved CPU state, normal execution continues.
// This method can be used to implement "multi threading", switching different
// processes with one another with regular intervals.
// CPU saved states may be modified, so previous execution may not be restored.
//
// The CPU refers to the IDT similarly to how it handles the GDT, storing a
// descriptor address in a special register with a special instruction.
//
// IDT anatomy: https://wiki.osdev.org/Interrupt_Descriptor_Table

mod entry;
mod descriptor;
mod isr;

use core::mem::zeroed;

use self::entry::IdtEntry;
use self::descriptor::IdtDescriptor;

/* ==== TYPE DEFINITION ===================================================== */
/// Wrapper type used to better implement methods related to the IDT.
pub struct Idt {
    /// List of IDT entries that associate interrupts and handler routines.
    entries: [IdtEntry; 256]
}

/* ==== CONSTRUCTOR AND METHODS ============================================= */
impl Idt {
    /// Initializes zeroed IDT array - all entries have present bit set to 0.
    const fn empty() -> Self {
        Self { entries: unsafe { zeroed() } }
    }

    /// Associates the given IDT entry to the given interrupt number.
    fn set(&mut self, interrupt: u8, entry: IdtEntry) {
        self.entries[interrupt as usize] = entry;
    }

    /// Returns a descriptor that contains this IDT's size and address.
    fn get_descriptor(&self) -> IdtDescriptor {
        IdtDescriptor::new(&self.entries)
    }
}

/* ==== IDT DATA and INITIALIZATION ========================================= */
/// Define IDT - No valid entries for now: present flags are initialized to 0
static mut IDT: Idt = Idt::empty();

/// Defines and loads IDT Descriptor, initialize IDT entries with ASM ISRs.
/// Descriptor doesn't need to be const: value is copied in the IDTR register.
pub fn init() {
    let descriptor: IdtDescriptor = unsafe { IDT.get_descriptor() };
    unsafe { core::arch::asm!( "lidt [eax]", in("eax") &descriptor ); }

    // Initialize IDT with ASM method pointers
    isr::init( unsafe { &mut IDT } );
}