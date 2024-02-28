use crate::pmio::Port;
use crate::pmio::PortMappedAddr;

/* ==== STATIC INITIALIZATION AND SYNCHRONIZATION =========================== */
/** Crate static VGA instance to access mutably using the public get_vga method.
 *! Using VGA is unsafe since it requires static multi-thread mutable access. */
static mut VGA: Vga = Vga::new();

/** Public method to get mutable reference to static VGA instance.
 *! Using VGA is unsafe since it requires static multi-thread mutable access. */
pub fn get_vga() -> &'static mut Vga { unsafe { &mut VGA } }

/* ==== TYPE DEFINITION ===================================================== */
pub struct Vga {
    /** This variable stores the current absolute VGA buffer position at
        which the characters will be printed.
        The value is initialized at VGA_BUFFER_START (0xB8000). */
    buffer_position: *mut u16,
    /** VGA port used to write register index. */
    register_index_w_3_port: Port,
    /** VGA port used to read/write register specified at the index port. */
    register_index_rw_3_port: Port
}

/* ==== TYPE CONSTANTS ====================================================== */
impl Vga {
    /** VGA text buffer start address; used to initialize buffer position. */
    const BUFFER_START: *const u16 = 0xB8000 as *const u16;
    /** VGA text buffer end address; used to check maximum buffer position. */
    const BUFFER_END: *const u16 = (0xB8000 + Vga::MAX_CHARACTERS_SCREEN*2 - 1) as *const u16;
    
    /** Maximum number of lines supported by VGA. */
    const MAX_LINES: usize = 25;
    /** Maximum number of characters per line supported by VGA. */
    const MAX_CHARACTERS_LINE: usize = 80;
    /** Maximum number of characters per screen supported by VGA. */
    const MAX_CHARACTERS_SCREEN: usize = Vga::MAX_CHARACTERS_LINE * Vga::MAX_LINES;
}

/*//! ==== WIP METHODS ====================================================== */
impl Vga {
    pub fn clear_cursor(&self) -> () {
        self.register_index_w_3_port.outb(0x0A);
        self.register_index_rw_3_port.outb(0x20);
    }
    // TODO: implement cursor handling methods

    pub fn clear_screen(&mut self) {
        self.clear();
        self.clear_cursor();
    }
}

/* ==== STATIC TYPE METHODS ================================================= */
impl Vga {
    /** Declare constructor as 'const' in order to declare static instances. */
    const fn new() -> Vga {
        Vga {
            buffer_position: Vga::BUFFER_START as *mut u16,
            register_index_w_3_port: unsafe { Port::new(PortMappedAddr::VgaRegisterIndexW3) },
            register_index_rw_3_port: unsafe { Port::new(PortMappedAddr::VgaRegisterIndexRW3) }
        }
    }
}

/* ==== PUBLIC TYPE METHODS ================================================= */
#[allow(dead_code)]
impl Vga {
    /** Writes a character at current VGA buffer position, incrementing it.
     *  TODO: handle special characters
     *  TODO: handle relative position > MAX_CHARACTERS (wrap? scroll?) */
    pub fn print_char(&mut self, ascii: u8, color: u8) {
        // Handle \n character: go to new line instead of printing.
        if ascii == b'\n' {
            self.line_feed();
            return;
        }

        if ascii == b'\r' {
            self.carriage_return();
            return;
        }

        // Concat color and ascii byte in character word
        let character: u16 = ((color as u16) * 256) + ascii as u16;
        unsafe { *self.buffer_position = character; }
        self.buffer_position = unsafe { self.buffer_position.add(1) };

        // Check if the buffer position is overflowing the maximum buffer size
        self.check_buffer_position();
    }

    /** Writes ASCII string at current VGA buffer position, incrementing it. */
    pub fn print(&mut self, s: &[u8]) -> () {
        for c in s.iter() {
            self.print_char(*c, 0x02);
        }
    }

    /** Prints string (see 'print()'), then moves buffer to new line. */
    pub fn println(&mut self, s: &[u8]) {
        self.print(s);
        self.carriage_return();
        self.line_feed();
    }

    pub fn ln(&mut self) {
        self.carriage_return();
        self.line_feed();
    }

    /** Clears the screen filling the VGA buffer with ' ' and black background.
     *  It also resets the buffer position to the starting position. */
    pub fn clear(&mut self) -> () {
        // Move buffer position at the start of the screen
        self.reset_buffer_position();

        // Fill the entire screen with space character and black background
        for _ in 0..Vga::MAX_CHARACTERS_SCREEN {
            self.print_char(b' ', 0x0F);
        }

        // Move buffer position at the start of the screen again
        self.reset_buffer_position();
    }

    pub fn clearln(&mut self) -> () {

        // Set buffer position to line start (save value for later)
        self.carriage_return();
        let t = self.buffer_position;

        // Fill the entire line with space character and black background
        for _ in 0..Vga::MAX_CHARACTERS_LINE {
            self.print_char(b' ', 0x0F);
        }

        // Reset buffer position to line start
        self.buffer_position = t;
        
    }
}

/* ==== PRIVATE TYPE METHODS ================================================ */
impl Vga {
    /** Calculates the buffer address position relative to the start of the screen. */
    fn get_buffer_relative_position(&self) -> usize {
        self.buffer_position as usize - Vga::BUFFER_START as usize
    }
    
    /** Resets the buffer_position value to VGA_BUFFER_START (0xB8000). */
    fn reset_buffer_position(&mut self) -> () {
        self.buffer_position = Vga::BUFFER_START as *mut u16;
    }

    /** If the buffer is overflowing, reset its position to the start. */
    fn check_buffer_position(&mut self) {
        if self.buffer_position as usize > Vga::BUFFER_END as usize {
            self.buffer_position = (self.buffer_position as usize - Vga::MAX_CHARACTERS_SCREEN*2) as *mut u16;
        }
    }

    /** Handles the Carriage Return special character (\r).
     *  It sets the buffer position to the start of the current line. */
    fn carriage_return(&mut self) {
        self.buffer_position = (self.buffer_position as usize - (self.get_buffer_relative_position() % (Vga::MAX_CHARACTERS_LINE*2))) as *mut u16;
    }
    /** Handles the Line Feed special character (\n).
     *  It sets the buffer position to the start of the next line. */
    fn line_feed(&mut self) {
        self.buffer_position = (self.buffer_position as usize + Vga::MAX_CHARACTERS_LINE*2) as *mut u16;
        self.check_buffer_position();
    }
}