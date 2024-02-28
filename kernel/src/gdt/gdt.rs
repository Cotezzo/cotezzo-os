use crate::{println, prints::{ToString, ToStringBase}, vga::get_vga};

/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct Gdt {
    pub limit_low: u16, // Limit (0-15 bits)
    pub base_low: u16,  // Base (0-15 bits)
    pub base_mid: u8,   // Base (16-23 bits)
    pub access: u8,     // Present, Ring, Type, Exec, Direct/Confirm, R/W, Accessed
    pub flags: u8,      // Granularity, Size, LongMode, Reserved, Limit (16-19 bits)
    pub base_high: u8   // Base (24-31 bits)
}

/* ==== CONSTRUCTOR ======================================================== */
impl Gdt {
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
impl Gdt {
    /** Defines a valid segment that can be accessed. */
    pub const ValidSegment: u8 = 0b1_0000000;

    /** Required privilege level to access segment - ring 0 (Kernel) */
    pub const PrivilegeLevelRing0: u8 = 0b0_00_00000;
    /** Required privilege level to access segment - ring 1 */
    pub const PrivilegeLevelRing1: u8 = 0b0_01_00000;
    /** Required privilege level to access segment - ring 2 */
    pub const PrivilegeLevelRing2: u8 = 0b0_10_00000;
    /** Required privilege level to access segment - ring 3 (Userland) */
    pub const PrivilegeLevelRing3: u8 = 0b0_11_00000;

    /** Defines a system segment */
    pub const TaskSegment: u8 = 0b000_0_0_000;
    /** Defines a data segment */
    pub const DataSegment: u8 = 0b000_1_0_000;
    /** Defines an executable code segment */
    pub const CodeSegment: u8 = 0b000_1_1_000;

    /** Data contained in the segment grows upwards */
    pub const DataDirectinUpwards: u8 = 0b00000_0_00;
    /** Data contained in the segment grows downwards */
    pub const DataDirectionDownwards: u8 =  0b00000_1_00;
    /** Code contained in the segment can be executed with lower privilege */
    pub const CodeLowerRingExecutionAllowed: u8 = 0b00000_1_00;

    /** Code contained in the segment can be read */
    pub const CodeReadAllowed: u8 = 0b000000_1_0;
    /** Data can be write to the data segment */
    pub const DataWriteAllowed: u8 = 0b000000_1_0;

    /** To be set used for read only segments, since CPU would try to set it 
        otherwise, causing a fault */
    pub const Accessed: u8 = 0b0000000_1;
}

/* ==== GDT FLAGS =========================================================== */
impl Gdt {
    /** Defines a valid segment that can be accessed. */
    pub const FlagGranularity: u8 = 0b1_0000000;
    /** Defines a 32bit segment */
    pub const Flag32BitSegment: u8 = 0b0_1_0_00000;
    /** Defines a 64bit segment */
    pub const Flag64BitSegment: u8 = 0b0_0_1_00000;
}

impl Gdt {
    pub fn print(&self) -> () {
        let limit_low = self.limit_low;
        let base_low = self.base_low;
        println!("limit_low: ", limit_low.to_string_base(2));
        println!("base_low: ", base_low.to_string_base(2));
        println!("base_mid: ", self.base_mid.to_string_base(2));
        println!("access: ", self.access.to_string_base(2));
        println!("flags: ", self.flags.to_string_base(2));
        println!("base_high: ", self.base_high.to_string_base(2));
    }
}