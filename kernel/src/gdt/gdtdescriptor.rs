use core::mem::size_of;
use super::gdt::Gdt;

/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct GdtDescriptor {
    pub size: u16,          // Total GDT size
    pub addr: *const u8//Addr
}

/* ==== CONSTRUCTOR ========================================================= */
impl GdtDescriptor {
    pub const fn new(gdt: &[Gdt]) -> Self {
        Self {
            size: (gdt.len() * size_of::<Gdt>()) as u16,
            addr: gdt as *const [Gdt] as *const u8
        }
    }
}