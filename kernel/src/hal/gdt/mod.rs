mod entry;
mod descriptor;
use entry::GdtEntry;
use self::descriptor::GdtDescriptor;

/* ==== ASM EXTERN METHODS ================================================== */
extern "C" {
    fn _c_load_gdt(descriptor: *const GdtDescriptor, code_selector: u16, data_selector: u16);
}

/* ==== TYPE DEFINITION ===================================================== */
/// Wrapper type used to better implement methods related to the GDT.
pub struct Gdt {
    /// List of GDT entries that associate memory segments to flags and rules.
    entries: [GdtEntry; 3]
}

/* ==== CONSTRUCTOR AND METHODS ============================================= */
/// Bitwise OR between given arguments
#[macro_export] macro_rules! or { ($($arg:expr),*) => { 0 $( | $arg )* }; }

impl Gdt {
    pub const CODE_SELECTOR: u16 = 1 * 8;
    pub const DATA_SELECTOR: u16 = 2 * 8;

    /// Initialized GDT entries with 32pm code and data segments.
    const fn new() -> Self {
        Self {
            entries: [
                // Null Table
                GdtEntry::new(0, 0, 0, 0),
            
                // 32pm code segment
                GdtEntry::new(0, 0xFFFFFFFF,
                    or!(GdtEntry::VALID_SEGMENT, GdtEntry::PRIVILEGE_LEVEL_RING0, GdtEntry::CODE_SEGMENT, GdtEntry::CODE_READ_ALLOWED),
                    or!(GdtEntry::FLAG_GRANULARITY, GdtEntry::FLAG_SEGMENT_32BIT)),
            
                // 32pm data segment
                GdtEntry::new(0, 0xFFFFFFFF,
                    or!(GdtEntry::VALID_SEGMENT, GdtEntry::PRIVILEGE_LEVEL_RING0, GdtEntry::DATA_SEGMENT, GdtEntry::DATA_WRITE_ALLOWED),
                    or!(GdtEntry::FLAG_GRANULARITY, GdtEntry::FLAG_SEGMENT_32BIT))
            ]
        }
    }

    /// Returns a descriptor that contains this GDT's size and address.
    fn get_descriptor(&self) -> GdtDescriptor {
        GdtDescriptor::new(&self.entries)
    }
}

/* ==== GDT DATA and INITIALIZATION ========================================= */
/// Define new GDT entries - we already defined these in bootloader data, but
/// that memory could be freed, so they are defined again in the kernel.
///! Needs to be const, not static - why? Idk...
const GDT: Gdt = Gdt::new();

/// Defines and loads GDT Descriptor and sets segments for 32 bit protected
/// flat memory model - the same already set in the stage-2, but free up the
/// bootloader memory could be needed, so they are defined again.
/// Other than that, we don't need the 16 bit real mode entries anymore.
/// Descriptor doesn't need to be const: value is copied in the IDTR register.
pub fn init() {
    let descriptor: GdtDescriptor = GDT.get_descriptor();
    unsafe { _c_load_gdt(&descriptor, Gdt::CODE_SELECTOR, Gdt::DATA_SELECTOR); }
}