use crate::hal::gdt;

/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,        // ISR address (0-15 bits)
    segment_selector: u16,  // GDT Code selector - 0x08
    reserved: u8,
    flags: u8,              // Gate Type (0-3 bits), 0, DPL (Descriptor
                            // Privilege Field, 5-6 bits), Present (must be 1)
    offset_high: u16        // ISR address (16-32 bits)
}

/* ==== CONSTRUCTOR ========================================================= */
impl IdtEntry {
    pub fn new(offset: unsafe extern "C" fn() -> (), gate_type: u8, ring: u8) -> Self {
        let offset: usize = unsafe { core::mem::transmute(offset) };
        let offset_low: u16 = (offset & 0xFFFF) as u16;
        let offset_high: u16 = (offset >> 16) as u16;
        let flags: u8 = 0b10000000 | ring | gate_type;
        Self {
            offset_low, offset_high,
            segment_selector: gdt::Gdt::CODE_SELECTOR,
            reserved: 0,
            flags
        }
    }
}

/* ==== IDT FLAGS =========================================================== */
#[allow(dead_code)] impl IdtEntry {
    /** To be used for hardware multitasking. */
    pub const FLAG_GATE_TASK: u8 = 0b0000_0101;
    /** 16rm - saves next instruction and continues normally after handling.
     *  Interrupt gates also disable other interrupts during execution. */
    pub const FLAG_GATE_INTERRUPT_16BIT: u8 = 0b0000_0110;
    /** 16rm - saves current instruction so it can be retried.
     *  Trap gates can be interrupted by another interrupt. */
    pub const FLAG_GATE_TRAP_16BIT: u8 = 0b0000_0111;
    /** 32pm - saves next instruction and continues normally after handling.
     *  Interrupt gates also disable other interrupts during execution. */
    pub const FLAG_GATE_INTERRUPT_32BIT: u8 = 0b0000_1110;
    /** 32pm - saves current instruction so it can be retried.
     *  Trap gates can be interrupted by another interrupt. */
    pub const FLAG_GATE_TRAP_32BIT: u8 = 0b0000_1111;

    /** Required privilege level to call the routine - ring 0 (Kernel) */
    pub const PRIVILEGE_LEVEL_RING0: u8 = 0b0_00_00000;
    /** Required privilege level to call the routine - ring 1 */
    pub const PRIVILEGE_LEVEL_RING1: u8 = 0b0_01_00000;
    /** Required privilege level to call the routine - ring 2 */
    pub const PRIVILEGE_LEVEL_RING2: u8 = 0b0_10_00000;
    /** Required privilege level to call the routine - ring 3 (Userland) */
    pub const PRIVILEGE_LEVEL_RING3: u8 = 0b0_11_00000;
}