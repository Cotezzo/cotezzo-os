use core::mem::size_of;

/* ==== ASM EXTERN METHODS ================================================== */
extern "C" {
    fn _c_load_gdt(descriptor: *const GdtDescriptor, code_selector: u16, data_selector: u16);
}

/* ==== TYPES DEFINITION ==================================================== */
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16, // Limit (0-15 bits)
    base_low: u16,  // Base (0-15 bits)
    base_mid: u8,   // Base (16-23 bits)
    access: u8,     // Present, Ring, Type, Exec, Direct/Confirm, R/W, Accessed
    flags: u8,      // Granularity, Size, LongMode, Reserved, Limit (16-19 bits)
    base_high: u8   // Base (24-31 bits)
}

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,          // Total GDT size
    addr: *const u8     // GDT address
}

/* ==== CONSTRUCTORS ======================================================= */
impl GdtEntry {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        let base_low: u16 = (base &  0x0000FFFF) as u16;
        let base_mid: u8 = ((base &  0x00FF0000) >> 16) as u8;
        let base_high: u8 = ((base & 0xFF000000) >> 24) as u8;

        let limit_low: u16 = (limit &  0x0000FFFF) as u16;
        let limit_high: u8 = ((limit & 0x000F0000) >> 16) as u8;
        let flags = (flags & 0xF0) | limit_high;

        Self { limit_low, base_low, base_mid, base_high, access, flags }
    }
}

impl GdtDescriptor {
    const fn new(gdt: &[GdtEntry]) -> Self {
        Self {
            size: (gdt.len() * size_of::<GdtEntry>() - 1) as u16,
            addr: gdt as *const [GdtEntry] as *const u8
        }
    }
}

/* ==== GDT ACCESS BYTE ===================================================== */
#[allow(dead_code)]
impl GdtEntry {
    /** Defines a valid segment that can be accessed. */
    const VALID_SEGMENT: u8 = 0b1_0000000;

    /** Required privilege level to access segment - ring 0 (Kernel) */
    const PRIVILEGE_LEVEL_RING0: u8 = 0b0_00_00000;
    /** Required privilege level to access segment - ring 1 */
    const PRIVILEGE_LEVEL_RING1: u8 = 0b0_01_00000;
    /** Required privilege level to access segment - ring 2 */
    const PRIVILEGE_LEVEL_RING2: u8 = 0b0_10_00000;
    /** Required privilege level to access segment - ring 3 (Userland) */
    const PRIVILEGE_LEVEL_RING3: u8 = 0b0_11_00000;

    /** Defines a system segment */
    const TASK_SEGMENT: u8 = 0b000_0_0_000;
    /** Defines a data segment */
    const DATA_SEGMENT: u8 = 0b000_1_0_000;
    /** Defines an executable code segment */
    const CODE_SEGMENT: u8 = 0b000_1_1_000;

    /** Data contained in the segment grows upwards */
    const DATA_DIRECTION_UPWARDS: u8 = 0b00000_0_00;
    /** Data contained in the segment grows downwards */
    const DATA_DIRECTION_DOWNWARDS: u8 =  0b00000_1_00;
    /** Code contained in the segment can be executed with lower privilege */
    const CODE_LOWER_RING_EXECUTION_ALLOWED: u8 = 0b00000_1_00;

    /** Code contained in the segment can be read */
    const CODE_READ_ALLOWED: u8 = 0b000000_1_0;
    /** Data can be write to the data segment */
    const DATA_WRITE_ALLOWED: u8 = 0b000000_1_0;

    /** To be set used for read only segments, since CPU would try to set it 
        otherwise, causing a fault */
    const ACCESSED: u8 = 0b0000000_1;
}

/* ==== GDT FLAGS =========================================================== */
#[allow(dead_code)]
impl GdtEntry {
    /** Defines a valid segment that can be accessed. */
    const FLAG_GRANULARITY: u8 = 0b1_0000000;
    /** Defines a 32bit segment */
    const FLAG_SEGMENT_32BIT: u8 = 0b0_1_0_00000;
    /** Defines a 64bit segment */
    const FLAG_SEGMENT_64BIT: u8 = 0b0_0_1_00000;
}

/* ==== GDT DATA and INITIALIZATION ========================================= */
/// Bitwise OR between given arguments
#[macro_export] macro_rules! or { ($($arg:expr),*) => { 0 $( | $arg )* }; }

/// Define new GDT entries - we already defined these in bootloader data, but
/// that memory could be freed, so they are defined again in the kernel.
const GDT: [GdtEntry; 3] = [
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
];

/// Define GDT Descriptor - constructor automatically sets GDT address and size
const DESCRIPTOR: GdtDescriptor = GdtDescriptor::new(&GDT);

pub const CODE_SELECTOR: u16 = 1 * 8;
pub const DATA_SELECTOR: u16 = 2 * 8;

/// Loads the defined GDT Descriptor and sets segments for 32 bit protected
/// flat memory model - the same already set in the stage-2, but free up
/// the bootloader memory could be needed, so they are defined again.
/// Other than that, we don't need the 16 bit real mode entries anymore.
pub fn init() {
    unsafe { _c_load_gdt(&DESCRIPTOR, CODE_SELECTOR, DATA_SELECTOR); }
}