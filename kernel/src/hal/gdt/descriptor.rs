use core::mem::size_of;
use crate::hal::gdt::entry::GdtEntry;

/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct GdtDescriptor {
    size: u16,          // Total GDT size
    addr: *const u8     // GDT address
}

/* ==== CONSTRUCTOR ========================================================= */
impl GdtDescriptor {
    pub const fn new(gdt: &[GdtEntry]) -> Self {
        Self {
            size: (gdt.len() * size_of::<GdtEntry>() - 1) as u16,
            addr: gdt as *const [GdtEntry] as *const u8
        }
    }
}