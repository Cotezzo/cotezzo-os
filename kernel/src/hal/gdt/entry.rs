/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16, // Limit (0-15 bits)
    base_low: u16,  // Base (0-15 bits)
    base_mid: u8,   // Base (16-23 bits)
    access: u8,     // Present, Ring, Type, Exec, Direct/Confirm, R/W, Accessed
    flags: u8,      // Granularity, Size, LongMode, Reserved, Limit (16-19 bits)
    base_high: u8   // Base (24-31 bits)
}

/* ==== CONSTRUCTOR ======================================================== */
impl GdtEntry {
    pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        let base_low: u16 = (base &  0x0000FFFF) as u16;
        let base_mid: u8 = ((base &  0x00FF0000) >> 16) as u8;
        let base_high: u8 = ((base & 0xFF000000) >> 24) as u8;

        let limit_low: u16 = (limit &  0x0000FFFF) as u16;
        let limit_high: u8 = ((limit & 0x000F0000) >> 16) as u8;
        let flags = (flags & 0xF0) | limit_high;

        Self { limit_low, base_low, base_mid, base_high, access, flags }
    }
}

/* ==== GDT ACCESS BYTE ===================================================== */
#[allow(dead_code)]
impl GdtEntry {
    /** Defines a valid segment that can be accessed. */
    pub const VALID_SEGMENT: u8 = 0b1_0000000;

    /** Required privilege level to access segment - ring 0 (Kernel) */
    pub const PRIVILEGE_LEVEL_RING0: u8 = 0b0_00_00000;
    /** Required privilege level to access segment - ring 1 */
    pub const PRIVILEGE_LEVEL_RING1: u8 = 0b0_01_00000;
    /** Required privilege level to access segment - ring 2 */
    pub const PRIVILEGE_LEVEL_RING2: u8 = 0b0_10_00000;
    /** Required privilege level to access segment - ring 3 (Userland) */
    pub const PRIVILEGE_LEVEL_RING3: u8 = 0b0_11_00000;

    /** Defines a system segment */
    pub const TASK_SEGMENT: u8 = 0b000_0_0_000;
    /** Defines a data segment */
    pub const DATA_SEGMENT: u8 = 0b000_1_0_000;
    /** Defines an executable code segment */
    pub const CODE_SEGMENT: u8 = 0b000_1_1_000;

    /** Data contained in the segment grows upwards */
    pub const DATA_DIRECTION_UPWARDS: u8 = 0b00000_0_00;
    /** Data contained in the segment grows downwards */
    pub const DATA_DIRECTION_DOWNWARDS: u8 =  0b00000_1_00;
    /** Code contained in the segment can be executed with lower privilege */
    pub const CODE_LOWER_RING_EXECUTION_ALLOWED: u8 = 0b00000_1_00;

    /** Code contained in the segment can be read */
    pub const CODE_READ_ALLOWED: u8 = 0b000000_1_0;
    /** Data can be write to the data segment */
    pub const DATA_WRITE_ALLOWED: u8 = 0b000000_1_0;

    /** To be set used for read only segments, since CPU would try to set it 
        otherwise, causing a fault */
    pub const ACCESSED: u8 = 0b0000000_1;
}

/* ==== GDT FLAGS =========================================================== */
#[allow(dead_code)]
impl GdtEntry {
    /** Defines a valid segment that can be accessed. */
    pub const FLAG_GRANULARITY: u8 = 0b1_0000000;
    /** Defines a 32bit segment */
    pub const FLAG_SEGMENT_32BIT: u8 = 0b0_1_0_00000;
    /** Defines a 64bit segment */
    pub const FLAG_SEGMENT_64BIT: u8 = 0b0_0_1_00000;
}