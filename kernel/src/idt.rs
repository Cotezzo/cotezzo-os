use core::mem::{size_of, zeroed};
use crate::gdt;

// Types of interrupts:
// - Exceptions: CPU errors during instructions execution
// - Hardware Interupts (or Interrupt Request, IRP): handled by CPU die or
//   Programmable Interrupt Controller, passed to CPU one by one.
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

/* ==== TYPES DEFINITION ==================================================== */
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,        // ISR address (0-15 bits)
    segment_selector: u16,  // GDT Code selector - 0x08
    reserved: u8,
    flags: u8,              // Gate Type (0-3 bits), 0, DPL (Descriptor
                            // Privilege Field, 5-6 bits), Present (must be 1)
    offset_high: u16        // ISR address (16-32 bits)
}

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,              // Total GDT size
    addr: *const u8         // GDT address
}

/* ==== CONSTRUCTORS ======================================================== */
impl IdtEntry {
    pub const fn new(offset: u32, gate_type: u8, ring: u8) -> Self {
        let offset_low: u16 = (offset & 0xFFFF) as u16;
        let offset_high: u16 = (offset >> 16) as u16;
        let flags: u8 = 0b10000000 | ring | gate_type;
        Self {
            offset_low, offset_high,
            segment_selector: gdt::CODE_SELECTOR,
            reserved: 0,
            flags
        }
    }
}

impl IdtDescriptor {
    const fn new(idt: &[IdtEntry]) -> Self {
        Self {
            size: (idt.len() * size_of::<IdtEntry>() - 1) as u16,
            addr: idt as *const [IdtEntry] as *const u8
        }
    }
}

/* ==== IDT FLAGS =========================================================== */
#[allow(dead_code)]
impl IdtEntry {
    /** To be used for hardware multitasking. */
    const FLAG_GATE_TASK: u8 = 0b0000_0101;
    /** 16rm - saves next instruction and continues normally after handling.
     *  Interrupt gates also disable other interrupts during execution. */
    const FLAG_GATE_INTERRUPT_16BIT: u8 = 0b0000_0110;
    /** 16rm - saves current instruction so it can be retried.
     *  Trap gates can be interrupted by another interrupt. */
    const FLAG_GATE_TRAP_16BIT: u8 = 0b0000_0111;
    /** 32pm - saves next instruction and continues normally after handling.
     *  Interrupt gates also disable other interrupts during execution. */
    const FLAG_GATE_INTERRUPT_32BIT: u8 = 0b0000_1110;
    /** 32pm - saves current instruction so it can be retried.
     *  Trap gates can be interrupted by another interrupt. */
    const FLAG_GATE_TRAP_32BIT: u8 = 0b0000_1111;

    /** Required privilege level to call the routine - ring 0 (Kernel) */
    const PRIVILEGE_LEVEL_RING0: u8 = 0b0_00_00000;
    /** Required privilege level to call the routine - ring 1 */
    const PRIVILEGE_LEVEL_RING1: u8 = 0b0_01_00000;
    /** Required privilege level to call the routine - ring 2 */
    const PRIVILEGE_LEVEL_RING2: u8 = 0b0_10_00000;
    /** Required privilege level to call the routine - ring 3 (Userland) */
    const PRIVILEGE_LEVEL_RING3: u8 = 0b0_11_00000;
}

/* ==== GDT DATA and INITIALIZATION ========================================= */
/// Define IDT - No valid entries for now: present flags are initialized to 0
const IDT: [IdtEntry; 256] = unsafe { zeroed() };

/// Define IDT Descriptor - constructor sets IDT address and size
const DESCRIPTOR: IdtDescriptor = IdtDescriptor::new(&IDT);

/// Loads the defined GDT Descriptor and sets segments for 32 bit protected
/// flat memory model - the same already set in the stage-2, but free up
/// the bootloader memory could be needed, so they are defined again.
/// Other than that, we don't need the 16 bit real mode entries anymore.
pub fn init() {
    unsafe { core::arch::asm!( "lidt [eax]", in("eax") &DESCRIPTOR ); }
}