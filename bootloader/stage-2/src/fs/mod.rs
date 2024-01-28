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

    /*  FileSystem physical info */
    drive_number: u8,
    //drive_type: u8,
    //max_cylinders: u16,
    max_heads: u8,
    max_sectors: u8,

    /*  FileSystem metadata, privately read and modified */
    boot_sector: BootSector,
    fat: [u8; 512*9],
    root: [DirectoryEntry; 224], // 32*224 byte
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
        println!("Type: ", drive_type, " - Cylinders: ", max_cylinders, " - Heads:", max_heads, " - Sectors: ", max_sectors);
        
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

        //> Debug logs
        //>let ptr = unsafe { core::slice::from_raw_parts(addr, 512) };
        //>println!("Buffer (", ptr.len()," / ", size_of::<BootSector>(), " bytes) value:\r\n", ptr);

        /* ==== FAT ========================================================= */
        // Get FAT start and size on the disk, translate LBA to CHS.
        // For now, buffer is hardcoded to handle 9 sectors (9*512 bytes).
        let lba: u16 = boot_sector.get_fat_offset();
        let count: u8 = boot_sector.get_fat_size() as u8;
        let (cylinder, head, sector) = FS::lba_to_chs(lba, max_sectors, max_heads);

        // Call BIOS to load FAT into memory at [&fat].
        // If the BIOS can't read from disk, panic.
        let fat: [u8; 512*9] = unsafe { zeroed() };          // TODO: optimise this
        let addr: *const u8 = &fat as *const u8;

        //> Debug logs
        let addr_str = addr.to_string_base(16);
        println!("LBA: ", lba, "  - CHS: ", cylinder, "/", head, "/", sector, " - Count: ", count, "  - Addr: ", addr_str, " < FAT");

        let outcome: bool = unsafe { _c_disk_read(drive_number, cylinder, head, sector, count, addr) };
        if !outcome { panic!("Could not read FAT!"); }

        /* ==== ROOT DIRECTORY ============================================== */
        // Root is right after the FATs, and it has N entries of 32 bytes each.
        // Assuming entries are 32 bytes, size in sectors is root_entries / 16.
        // Entries: 224, Size 32B, Total Sector Size: 14
        // 224*32 / 512 = 224 / 16 = 14        32 == size_of::<DirectoryEntry>()
        // For now, buffer is hardcoded to handle 224 entries (224*32 bytes).
        let lba: u16 = boot_sector.get_root_offset();
        let count: u8 = boot_sector.get_root_size() as u8;
        let (cylinder, head, sector) = FS::lba_to_chs(lba, max_sectors, max_heads);

        // Call BIOS to load FAT into memory at [&fat].
        // If the BIOS can't read from disk, panic.
        let root: [DirectoryEntry; 224] = unsafe { zeroed() };   // TODO: optimise this
        let addr: *const u8 = &root as *const DirectoryEntry as *const u8;

        //> Debug logs
        let addr_str = addr.to_string_base(16);
        println!("LBA: ", lba, " - CHS: ", cylinder, "/", head, "/", sector, " - Count: ", count, " - Addr: ", addr_str, " < Root Directory");

        let outcome: bool = unsafe { _c_disk_read(drive_number, cylinder, head, sector, 14, addr) };
        if !outcome { panic!("Could not read root directory!"); }
        
        // Return FS instance ownership
        FS {
            drive_number,
            //drive_type, max_cylinders,
            max_heads, max_sectors,
            boot_sector, fat, root
        }
    }
}

/* ==== TYPE METHODS ======================================================== */
impl FS {

    /** Searches for the given file starting from the root directory and
     *  initializes struct with metadata, without reading from disk.
     *  TODO: read sub-directories
     *  TODO: implement disk WRITE */
    pub fn file_open(&self, name: &[u8]) -> Option<File> {
        let entry: &DirectoryEntry = self.root_entry_read(name)?;
        Some( File::new(entry) )
    }

    pub fn file_read<'fs>(&self, file: &mut File) -> () {
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
            let (cylinder, head, sector) = FS::lba_to_chs(lba, self.max_sectors, self.max_heads);

            // As the LBA skips already read sectors, the count also decreases
            // If the leftover buffer is smaller than the current count, only
            // read what fits in.
            let count: u16 = cluster_size - current_cluster_read_sectors;
            let count: u16 = core::cmp::min(count, buffer_capacity as u16);

            //> Debug logs
            //>let addr_str = addr.to_string_base(16);
            //>let file_name = file.metadata.name.as_slice();
            //>println!("LBA: ", lba, " - CHS: ", cylinder, "/", head, "/", sector, " - Count: ", count, " - Addr: ", addr_str, " < ", file_name);

            // Write at load_addr disk data from given offset (Root Directory)
            let outcome: bool = unsafe { _c_disk_read(self.drive_number, cylinder, head, sector, count as u8, addr) };
            if !outcome { panic!("Could not read file content!"); }

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
    pub fn fat_entry_read(&self, cluster: u16) -> Option<u16> {

        // Get single byte position and find index array (element = 2B)
        let i: usize = cluster as usize * 3 / 2;

        // Check if the requested cluster can exist
        if i >= self.fat.len() { panic!("FAT cluster out-of-bounds!"); }

        // Get 4 if the reminder is 1 (odd number), 0 otherwise (even number)
        // This number is used for bitshifting by half byte
        let c: u16 = ((cluster * 3) % 2) * 4;
        
        // First element contains the least significant byte
        // If the reminder is odd, we only need the upper 4 bits
        let lsb: u8 = self.fat.get(i)? & (0xFF << c);

        // Second element contains the most significant byte
        // If the reminder is even, we only need the lower 4 bits
        let msb: u8 = self.fat.get(i+1)? & (0xFF >> (4-c));

        // "Concat" the two bytes in a word
        let word: u16 = ((msb as u16) * 256) + lsb as u16;

        // If the reminder is odd, the entry is in the upper 12bits, right shift
        // If the reminder is even, we need to remove the upper 4bits
        Some((word >> c) & 0x0FFF)
    }

    fn root_entry_read(&self, name: &[u8]) -> Option<&DirectoryEntry> {
        for i in 0..self.root.len() {
            // Get ith entry in the directory
            let entry: &DirectoryEntry = self.root.get(i)?;

            // If the first byte is NULL, the previous entry was the last one
            if *entry.name.get(0)? == 0x00 { break; }

            // If the name is equal to the input, this is the entry
            if name.eq(&entry.name) { return Some(entry); }
        }
        None
    }
}