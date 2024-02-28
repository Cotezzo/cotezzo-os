use core::mem::size_of;

use crate::println;
use crate::prints::ToString;
use crate::prints::ToStringBase;
use crate::vga::get_vga;
use crate::Gdt;
use crate::GdtDescriptor;

pub mod gdt;
pub mod gdtdescriptor;

/* ==== ASM EXTERN METHODS ================================================== */
extern "C" {
    fn _c_load_gdt(descriptor: *const GdtDescriptor, code_entry_offset: u16, data_entry_offset: u16);
}

/* ==== GDT DATA ============================================================ */
/// Bitwise OR between given arguments
#[macro_export] macro_rules! or { ($($arg:expr),*) => { 0 $( | $arg )* }; }

/// Define new GDT entries - we already defined these in bootloader data, but
/// that memory could be freed, so they are defined again in the kernel.
const GDT: [Gdt; 3] = [
    // Null Table
    Gdt::new(0, 0, 0, 0),

    // 32pm code segment
    Gdt::new(0, 0xFFFFFFFF,
        or!(Gdt::ValidSegment, Gdt::PrivilegeLevelRing0, Gdt::CodeSegment, Gdt::CodeReadAllowed),
        or!(Gdt::FlagGranularity, Gdt::Flag32BitSegment)),

    // 32pm data segment
    Gdt::new(0, 0xFFFFFFFF,
        or!(Gdt::ValidSegment, Gdt::PrivilegeLevelRing0, Gdt::DataSegment, Gdt::DataWriteAllowed),
        or!(Gdt::FlagGranularity, Gdt::Flag32BitSegment))
];

const GDT_DESCRIPTOR: GdtDescriptor = GdtDescriptor::new(&GDT);

const GDT_CODE_ENTRY_INDEX: u16 = 1;
const GDT_DATA_ENTRY_INDEX: u16 = 2;

pub fn init_gdt() {
    unsafe {
        _c_load_gdt(&GDT_DESCRIPTOR, GDT_CODE_ENTRY_INDEX*8, GDT_DATA_ENTRY_INDEX*8);
    }
}