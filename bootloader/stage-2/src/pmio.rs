use core::arch::asm;

/* ==== PORT MAPPED ADDRESSES =============================================== */
#[allow(dead_code)]
/** Enum defining the Port Mapped I/O memory addresses of each (used) port.
 *  These addresses doesn't refer to the RAM, but to a dedicated port memory. */
 pub enum PortMappedAddr {
    // ...
    VgaRegisterIndexW1 = 0x3C4,
    VgaRegisterIndexRW1,    // Previous +1
    VgaDACMaskRegister = 0x3C6,
    VgaRegisterIndexW3 = 0x3D4,
    VgaRegisterIndexRW3,
    VgaRegisterIndexW2 = 0x3CE,
    VgaRegisterIndexRW2,    // Previous +1
    // ...
}

/* ===== TYPE DEFINITION ==================================================== */
pub struct Port {
    port: u16
}

/* ===== STATIC TYPE METHODS ================================================ */
impl Port {
    /** Creates a new Port instance for the specified Address.
     *! Marked as unsafe since using Port I/O might result in unexpected
     *! behaviour. This responsability is given to the constructor caller. */
    pub const unsafe fn new(port: PortMappedAddr) -> Port {
        Port { port: port as u16 }
    }
}

/* ===== PUBLIC TYPE METHODS ================================================ */
/*  Declare ASM instruction to be executed.
    "in" puts the variable into the DX register.
    "out" stores the register into variable after execution.
    "inout" puts the value into the AL register; since the 'out' instruction
    could modify AL's value, using the "inout" and discarding the final
    value (=> _) clarifies the expected behaviour. */
#[allow(dead_code)]
impl Port {
    /** Sends the input u8 value to the Port. */
    pub fn outb(&self, value: u8) {
        unsafe { asm!( "out dx, al", in("dx") self.port, inout("al") value => _ ); }
    }
    /** Sends the input u16 value to the Port. */
    pub fn outw(&self, value: u16) {
        unsafe { asm!( "out dx, ax", in("dx") self.port, inout("ax") value => _ ); }
    }
    /** Sends the input u32 value to the Port. */
    pub fn outl(&self, value: u32) {
        unsafe { asm!( "out dx, eax", in("dx") self.port, inout("eax") value => _ ); }
    }

    /** Reads an u8 value from the Port. */
    pub fn inb(&self) -> u8 {
        let value: u8;
        unsafe { asm!( "in al, dx", in("dx") self.port, out("al") value ); }
        value
    }
    /** Reads an u16 value from the Port. */
    pub fn inw(&self) -> u16 {
        let value: u16;
        unsafe { asm!( "in ax, dx", in("dx") self.port, out("ax") value ); }
        value
    }
    /** Reads an u32 value from the Port. */
    pub fn inl(&self) -> u32 {
        let value: u32;
        unsafe { asm!( "in eax, dx", in("dx") self.port, out("eax") value ); }
        value
    }
}