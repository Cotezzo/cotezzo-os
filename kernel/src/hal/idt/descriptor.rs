use core::mem::size_of;
use crate::hal::idt::IdtEntry;

/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct IdtDescriptor {
    size: u16,              // Total IDT size
    addr: *const u8         // IDT address
}

/* ==== CONSTRUCTOR ========================================================= */
impl IdtDescriptor {
    pub const fn new(idt: &[IdtEntry]) -> Self {
        Self {
            size: (idt.len() * size_of::<IdtEntry>() - 1) as u16,
            addr: idt as *const [IdtEntry] as *const u8
        }
    }
}