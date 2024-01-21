use core::{mem::size_of, panic};


extern "C" {
    pub fn _c_disk_reset(drive: u8) -> bool;
    pub fn _c_disk_read(drive: u8, cylinder: u16, head: u8, sector: u8, count: u8, address: *const u16) -> bool;
    pub fn _c_disk_get_params(drive: u8, drive_type_out: *const u8, cylinders_out: *const u16, heads_out: *const u8, sectors_out: *const u8) -> bool;
}

/* ==== STRUCTS ============================================================= */
/* Public struct used from the main module */
 #[derive(Debug)]
pub struct DiskFat12 {
    boot_sector: &'static mut BootSector,
    fat: Fat,
    root_directory: Directory
}

/*  Define FAT12 headers and bootloader sector.
    All the header values are mapped, but the bootloader code is ignored. */
 #[repr(C, packed)]
 #[derive(Debug)]
pub struct BootSector {
    // BIOS Parameter Block
    jump_instruction: [u8; 3],
    oem_id: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    root_entries : u16,
    sector_count: u16,
    media_descriptor: u8,
    sectors_per_fat: u16,
    sectors_per_cylinder: u16,
    heads_count: u16,
    hidden_sectors_count: u32,
    large_sector_count: u32,

    // Extended Boot Record
    drive_number: u8,
    reserved: u8,
    volume_id : u32,
    volume_label : [u8; 11],
    system_id: [u8; 8]

    // BootLoader code (ignored)
}

#[derive(Debug)]
pub struct Fat {
    // ! Unsafe if memory allocation is not handled, memory can be overwritten
    entries: *mut u8,
    entries_count: u16
}

#[repr(C, packed)]
#[derive(Debug)]
struct DirectoryEntry {
    name: [u8; 11],
    attributes: u8,             // READ_ONLY=0x01 HIDDEN=0x02 SYSTEM=0x04 VOLUME_ID=0x08 DIRECTORY=0x10 ARCHIVE=0x20 LFN=READ_ONLY|HIDDEN|SYSTEM|VOLUME_ID (LFN means that this entry is a long file name entry)
    reserved: u8,
    creation_time_tenths: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    upper_first_cluster: u16,
    last_change_time: u16,
    last_change_date: u16,
    lower_first_cluster: u16,
    file_size: u32
}   // 32 byte

#[derive(Debug)]
pub struct Directory {
    // ! Unsafe if memory allocation is not handled, memory can be overwritten
    entries: *mut DirectoryEntry,
    entries_count: u16
}

/* ==== STRUCT IMPLEMENTATIONS ============================================== */
impl DiskFat12 {
    pub fn new(mut load_addr: usize) -> DiskFat12 {

        /* ==== BOOT SECTOR ==== */
        // Get buffer size - fixed, known at compile time knowing the type
        // Size known at compile time: create plain array allocated in the stack
        let boot_sector_size = size_of::<BootSector>();
        let boot_sector: *mut BootSector = load_addr as *mut BootSector;

        // Write at load_addr disk data from start (Boot Sector)
        //_disk_read(drive, cylinder, head, sector, count, address);
        // TODO: call _disk_read at 0
        
        // Dereference pointer to use the loaded value
        let boot_sector: &mut BootSector = unsafe { &mut *boot_sector };

        /* ==== FAT ==== */
        // Update loading address to next byte after boot_sector
        load_addr += boot_sector_size;

        // Calculate FAT offset and size using boot sector data
        let fat_start: u16 = boot_sector.get_fat_start();
        let fat_size: u16 = boot_sector.get_fat_size();
        let fat_pointer: *mut u8 = load_addr as *mut u8;

        // Write at load_addr disk data from given offset (FAT)
        // TODO: call _disk_read at fat_start

        // Wrap FAT data pointer in data structure
        let fat: Fat = Fat {
            entries: fat_pointer,
            entries_count: fat_size
        };

        /* ==== ROOT DIRECTORY ==== */
        // Update loading address to next byte after boot_sector
        load_addr += fat_size as usize;

        // Calculate root directory offset and size using boot sector data
        let root_start: u16 = boot_sector.get_root_start();
        let root_size: usize = boot_sector.get_root_size();
        let root_entries_count: u16 = boot_sector.root_entries;
        let root_pointer: *mut DirectoryEntry = load_addr as *mut DirectoryEntry;

        // Write at load_addr disk data from given offset (Root Directory)
        // TODO: call _disk_read at root_start

        // Wrap entries in Directory data structure
        let root_directory: Directory = Directory {
            entries: root_pointer,
            entries_count: root_entries_count
        };

        DiskFat12 { boot_sector, fat, root_directory }
    }
}

impl BootSector {
    pub fn get_fat_start(&self) -> u16 { self.reserved_sectors * self.bytes_per_sector }
    pub fn get_fat_size(&self) -> u16 { self.sectors_per_fat * self.bytes_per_sector }
    pub fn get_root_start(&self) -> u16 { self.get_fat_start() + (self.get_fat_size() * self.fat_count as u16) }
    pub fn get_root_size(&self) -> usize { self.root_entries as usize * size_of::<DirectoryEntry>() }
    pub fn get_cluster_region_start(&self) -> usize { self.get_root_start() as usize + self.get_root_size() }
    pub fn get_cluster_size(&self) -> usize { self.sectors_per_cluster as usize * self.bytes_per_sector as usize }
    pub fn get_cluster_start(&self, cluster: u16) -> usize {
        self.get_cluster_region_start() + (self.get_cluster_size() * (cluster - 2) as usize)
    }
}

impl Fat {
    pub fn get_entry(&self, cluster: usize) -> u16 {

        // Get single byte position and find index array (element = 2B)
        let i: usize = cluster * 3 / 2;

        // Check if the requested cluster can exist
        if i >= self.entries_count as usize {
            panic!("Requested FAT cluster is out-of-bounds!");
        }

        // Get 4 if the reminder is 1 (odd number), 0 otherwise (even number)
        // This number is used for bitshifting by half byte
        let c: usize = ((cluster * 3) % 2) * 4;
        
        // First element contains the least significant byte
        // If the reminder is odd, we only need the upper 4 bits
        let lsb: u8 = unsafe { *(self.entries.add(i)) } & (0xFF << c);

        // Second element contains the most significant byte
        // If the reminder is even, we only need the lower 4 bits
        let msb: u8 = unsafe { *(self.entries.add(i+1)) } & (0xFF >> (4-c));

        // "Concat" the two bytes in a word
        let word: u16 = ((msb as u16) * 256) + lsb as u16;

        // If the reminder is odd, the entry is in the upper 12bits, right shift
        // If the reminder is even, we need to remove the upper 4bits
        (word >> c) & 0x0FFF
    }
}

impl Directory {
    pub fn get_entry(&self, name: &str) -> Option<&DirectoryEntry> {
        for i in 0..self.entries_count as usize {
            // Get ith entry in the directory
            // Dereference from pointer and get readonly reference
            let entry: &DirectoryEntry = unsafe { &*(self.entries.add(i)) };

            //dbg!(String::from_utf8(Vec::from(entry.name)));
            //dbg!(entry.lower_first_cluster);

            // If the first byte is NULL, the previous entry was the last one
            if *entry.name.get(0)? == 0x00 { break; }

            // If the name is equal to the input, this is the entry
            if name.as_bytes().eq(&entry.name) { return Some(&entry); }
        }
        None
    }
}

/* ==== METHODS ============================================================= */
impl DiskFat12 {
    pub fn read_entry_content(&self, mut load_addr: usize, entry: &DirectoryEntry) -> () {

        // Get the first cluster the data is stored in from the entry
        let mut current_cluster: u16 = entry.lower_first_cluster;

        // Get the size of the disk data that needs to be read
        let cluster_size: usize = self.boot_sector.get_cluster_size();

        loop {
            // Get offset of the given cluster in the disk
            let cluster_offset_start: usize = self.boot_sector.get_cluster_start(current_cluster);

            // Write at load_addr disk data from given offset (Root Directory)
            // TODO: call _disk_read at cluster_offset_start

            // Check the FAT for the next cluster
            current_cluster = self.fat.get_entry(current_cluster as usize);

            // If the cluster number is higher than FF8, that was the last cluster
            if current_cluster >= 0x0FF8 { break; }

            // Update loading address to next byte after previous cluster
            load_addr += cluster_size;
        }
    }
}