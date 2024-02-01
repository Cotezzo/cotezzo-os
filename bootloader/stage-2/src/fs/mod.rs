use core::mem::zeroed;
use crate::prints::ToString;
use crate::{println, prints::ToStringBase};
use crate::vga::get_vga;
use self::file::File;
use self::{bootsector::BootSector, directory::DirectoryEntry};

/* ==== MODULE EXPORTS ====================================================== */
pub mod bootsector;
pub mod directory;
pub mod file;

/* ==== ASM EXTERN METHODS ================================================== */
extern "C" {
    pub fn _c_disk_reset(drive: u8) -> bool;
    pub fn _c_disk_read(drive: u8, cylinder: u16, head: u8, sector: u8, count: u8, addr: *const u8) -> bool;
    pub fn _c_disk_get_params(drive: u8, drive_type: *const u8, max_cylinders: *const u16, max_heads: *const u8, max_sectors: *const u8) -> bool;
}

/* ==== TYPE DEFINITION ===================================================== */
/* Public struct used from the main module */
pub struct FS {

    /*  Physical drive informations */
    drive_number: u8,
    //drive_type: u8,
    //max_cylinders: u16,
    max_heads: u8,
    max_sectors: u8,

    /*  FileSystem informations */
    boot_sector: BootSector,

    // TODO: remove and use buffers
    //fat: [u8; 512*9],
    //root: [DirectoryEntry; 224], // 32*224 byte

    /*  FAT and Root Directory buffers - only store one sector (512 bytes)
        at a time, no need to always store the whole thing.
        The kernel is probably the only file in the disk anyway.
        The "sector" property indicates which sector of the FAT or the Root
        Directory is stored in the buffer - if another sector need to be
        accessed, it must be read from disk and stored in the buffer first. */
    fat_buffer: [u8; 512],
    root_buffer: [DirectoryEntry; 16],
    fat_sector: usize,
    root_sector: usize
}

/* ==== STATIC TYPE METHODS ================================================= */
impl FS {
    fn lba_to_chs(lba: u16, max_sectors: u8, max_heads: u8) -> (u16, u8, u8) {
        let max_sectors = max_sectors as u16;
        let max_heads = max_heads as u16;
        (
            (lba / max_sectors) / max_heads,            // C
            ((lba / max_sectors) % max_heads) as u8,    // H
            ((lba % max_sectors) + 1) as u8             // S
        )
    }

    pub fn new(drive_number: u8) -> FS {

        /* ==== DISK PARAMETERS ============================================= */
        // Initialize variables and get disk parameters:
        // _c_disk_get_params is implemented in ASM, it switches to 16rm,
        // calls BIOS INT 13,8 to retrieve disk data and returns to 32pm.
        // If the BIOS can't retrieve the data, return None.
        let drive_type: u8 = 0;
        let max_cylinders: u16 = 0;
        let max_heads: u8 = 0;
        let max_sectors: u8 = 0;
        let outcome: bool = unsafe { _c_disk_get_params(drive_number, &drive_type, &max_cylinders, &max_heads, &max_sectors) };
        if !outcome { panic!("Could not read disk parameters!"); }

        //> Debug logs
        println!("Number: ", drive_number, " - Type: ", drive_type, " - Cylinders: ", max_cylinders, " - Heads: ", max_heads, " - Sectors: ", max_sectors);
        
        /* ==== BOOT SECTOR ================================================= */
        // For the boot sector, read one sector (count=1) at disk start (lba=0)
        let lba: u16 = 0;
        let count: u8 = 1;

        // Boot sector is at first disk partition: LBA is 0.
        // Translate LBA to CHS for BIOS call after getting disk parameters.
        let (cylinder, head, sector) = FS::lba_to_chs(lba, max_sectors, max_heads);

        // Initialize boot sector struct zeroed (empty).
        let boot_sector: BootSector = unsafe { zeroed() };
        let addr: *const u8 = &boot_sector as *const BootSector as *const u8;

        //> Debug logs
        let addr_str = addr.to_string_base(16);
        println!("LBA: ", lba, "  - CHS: ", cylinder, "/", head, "/", sector, " - Count: ", count, "  - Addr: ", addr_str, " < Boot Sector");

        // Call BIOS to load boot sector into memory at [&boot_sector].
        // The boot sector is only 1 sector wide (512 bytes), so count = 1.
        // If the BIOS can't read from disk, panic.
        // TODO: before panicking, try resetting and retrying up to 3 times
        let outcome: bool = unsafe { _c_disk_read(drive_number, cylinder, head, sector, count, addr) };
        if !outcome { panic!("Could not read boot sector!"); }

        //*! FAT and Root Directory buffers are not initialized, data is read
        //*! from the disk when fat_buffer_read and root_buffer_read are called.

        /* ==== RETURN ====================================================== */
        // Return FS instance ownership
        FS {
            drive_number,
            //drive_type, max_cylinders,
            max_heads, max_sectors,

            boot_sector,
            fat_buffer: unsafe { zeroed() }, fat_sector: 0, 
            root_buffer: unsafe { zeroed() }, root_sector: 0
        }
    }
}

/* ==== TYPE METHODS ======================================================== */
impl FS {

    /* ==== FILES =========================================================== */
    /** Searches for the given file starting from the root directory and
        returns a File instance with a copy of its metadata.
        Searching for the file name could involve reading more root directory
        entries than are buffered (if any is buffered), so reading from disk
        and updating stored data might happen (hence, the mutable reference).
     *  TODO: read sub-directories
     *  TODO: implement disk WRITE */
    pub fn file_open(&mut self, name: &[u8]) -> Option<File> {
        let entry: &DirectoryEntry = self.root_entry_read(name)?;
        Some( File::new(entry.clone()) )
    }

    /** Fills the File buffer with its actual content read from the disk.
        If the File cluster info are present, they're used to load the next
        data to the buffer, overriding the previous content.
        Reading file content could involve following the FAT for cluster lookup
        in non buffered entries (if any is buffered), so reading from disk and
        updating stored data might happen (hence, the mutable reference). */
    pub fn file_read(&mut self, file: &mut File) -> () {
        // Get the first cluster the data is stored in from the entry.
        // This cluster number already accounts for the two empty FAT entries.
        let mut current_cluster: u16 = file.current_cluster;

        // Get the size of the disk data that needs to be read
        let cluster_size: u16 = self.boot_sector.get_cluster_size();

        // To keep track of the sectors read for each cluster
        let mut current_cluster_read_sectors: u16 = file.current_cluster_read_sectors;

        // Get content buffer raw pointer and maximum size
        let mut addr: *const u8 = &file.buffer as *const u8;
        let mut buffer_capacity: u16 = (file.buffer.len() / File::SECTION_SIZE) as u16;

        loop {
            // Get offset of the given cluster in the disk
            let cluster_offset_start: u16 = self.boot_sector.get_cluster_offset(current_cluster);

            // Get address of next sector to read, usually first cluster sector
            // If we already read some sectors of this cluster, skip those
            let lba = cluster_offset_start + current_cluster_read_sectors;

            // As the LBA skips already read sectors, the count also decreases
            // If the leftover buffer is smaller than the current count, only
            // read what fits in.
            let count: u16 = cluster_size - current_cluster_read_sectors;
            let count: u16 = core::cmp::min(count, buffer_capacity as u16);

            // Write at load_addr disk data from given offset (Root Directory)
            self.read_disk(lba, count as u8, addr, file.metadata.name.as_slice());

            // After reading, update read sectors and leftover buffer capacity
            buffer_capacity -= count;
            current_cluster_read_sectors += count;
            addr = unsafe { addr.add(count as usize * File::SECTION_SIZE) };

            // If the cluster has been fully read, read next cluster:
            // reset read sectors count, retrieve next cluster from FAT
            if cluster_size - current_cluster_read_sectors == 0 {
                current_cluster_read_sectors = 0;

                // Check the FAT for the next cluster
                // If there's none, treat it as last cluster (shouldn't happen)
                current_cluster = match self.fat_entry_read(current_cluster) {
                    Some(c) => c,
                    None => 0x0FF8
                };
            }

            // If the buffer is full, save reading metadata and exit
            if buffer_capacity == 0 {
                file.current_cluster = current_cluster;
                file.current_cluster_read_sectors = current_cluster_read_sectors;
                //>println!("Buffer is full!");
                break;
            }

            // If cluster number is >= FF8, that was the last cluster
            // Reset file read metadata and exit
            if current_cluster >= 0x0FF8 {
                file.current_cluster = file.metadata.lower_first_cluster;
                file.current_cluster_read_sectors = 0;
                break;
            }
        }
    }
}

/* ==== PRIVATE TYPE METHODS ================================================ */
impl FS {

    /* ==== FATs ============================================================ */
    fn fat_entry_read(&mut self, cluster: u16) -> Option<u16> {

        // Get single byte position and find index array (element = 2B)
        let i: usize = cluster as usize * 3 / 2;

        // Check if the requested cluster can exist
        if i >= self.fat_buffer.len() { panic!("FAT cluster out-of-bounds!"); }

        // Get 4 if the reminder is 1 (odd number), 0 otherwise (even number)
        // This number is used for bitshifting by half byte
        let c: u16 = ((cluster * 3) % 2) * 4;
        
        // First element contains the least significant byte
        // If the reminder is odd, we only need the upper 4 bits
        let lsb: u8 = self.fat_buffer_read(i)? & (0xFF << c);

        // Second element contains the most significant byte
        // If the reminder is even, we only need the lower 4 bits
        let msb: u8 = self.fat_buffer_read(i+1)? & (0xFF >> (4-c));

        // "Concat" the two bytes in a word
        let word: u16 = ((msb as u16) * 256) + lsb as u16;

        // If the reminder is odd, the entry is in the upper 12bits, right shift
        // If the reminder is even, we need to remove the upper 4bits
        Some((word >> c) & 0x0FFF)
    }

    fn fat_buffer_read(&mut self, entry_index: usize) -> Option<&u8> {
        let max_entry: usize = self.fat_sector * 512;
        let min_entry: usize = max_entry.saturating_sub(512 - 1);

        // If entry is out of range, buffer correct entries before reading.
        if entry_index+1 < min_entry || max_entry < entry_index+1 {
            
            // Calculate the sector (from root start) the entry is in.
            // Add te number to LBA to read only the needed sector.
            // Call BIOS to load root entries into memory at [&root_buffer].
            self.fat_sector = entry_index / 512 + 1;
            let lba: u16 = self.boot_sector.get_fat_offset() + self.fat_sector as u16 - 1;
            self.read_disk(lba, 1, &self.fat_buffer as *const u8, b"FAT");
        }

        // Get relative entry index (0-223 --> 0-15) and read from buffer.
        self.fat_buffer.get(entry_index % 512)
    }

    /* ==== DIRECTORIES ===================================================== */
    const ENTRIES_PER_SECTOR: usize = 512 / 32;
    /** Searches for the entry with the provided name in the root directory.
        If the needed entry is not currently loaded to the root directory
        buffer, a disk read is performed to load the correct data. */
    fn root_entry_read(&mut self, name: &[u8]) -> Option<&DirectoryEntry>{

        // Start looping for each root directory entry
        for entry_index in 0..self.boot_sector.root_entries as usize {

            // Calculate current buffered entries first and last actual index.
            // If there is not root_sector stored, both would be 0.
            //* Ex: root_sector = 0     max=0*16 -> 0       min=0-(16-1) -> 0
            //* Ex: root_sector = 1     max=1*16 -> 16      min=16-(16-1) -> 1
            //* Ex: root_sector = 2     max=2*16 -> 32      min=32-(16-1) -> 17
            // Entries 0-0: entry always out of range (empty buffer), read disk.
            // Entries 1-16 (-1 --> 0-15) are buffered and ready to be read.
            // Entries 17-32 (-1 --> 16-31) are buffered and ready to be read.
            let max_entry: usize = self.root_sector * Self::ENTRIES_PER_SECTOR;
            let min_entry: usize = max_entry.saturating_sub(Self::ENTRIES_PER_SECTOR - 1);

            // If entry is out of range, buffer correct entries before reading.
            if entry_index+1 < min_entry || max_entry < entry_index+1 {
                
                // Calculate the sector (from root start) the entry is in.
                // Add te number to LBA to read only the needed sector.
                // Call BIOS to load root entries into memory at [&root_buffer].
                self.root_sector = entry_index / 16 + 1;
                let lba: u16 = self.boot_sector.get_root_offset() + self.root_sector as u16 - 1;
                self.read_disk(lba, 1, &self.root_buffer as *const DirectoryEntry as *const u8, b"Root Directory");
            }

            // Get relative entry index (0-223 --> 0-15) and read from buffer.
            let entry: &DirectoryEntry =  self.root_buffer.get(entry_index % Self::ENTRIES_PER_SECTOR)?;

            // If name's first byte is NULL, there are no more entries, exit.
            if *entry.name.get(0)? == 0x00 { break; }

            // If the name matches the input, this is the entry, return it.
            if name.eq(&entry.name) { return Some(entry); }
        }

        // The file has not been found, return None.
        None
    }

    /* ==== I/O ============================================================= */
    /** Uses disk metadata to load data from disk at given memory location. */
    fn read_disk(&self, lba: u16, count: u8, addr: *const u8, reason: &[u8]) {
        let (cylinder, head, sector) = FS::lba_to_chs(lba, self.max_sectors, self.max_heads);

        //> Debug logs
        let addr_str = addr.to_string_base(16);
        println!("LBA: ", lba, " - CHS: ", cylinder, "/", head, "/", sector, " - Count: ", count, " - Addr: ", addr_str, " < ", reason);

        let outcome: bool = unsafe { _c_disk_read(self.drive_number, cylinder, head, sector, count, addr) };
        if !outcome { panic!("Could not read from disk!"); }
    }
}