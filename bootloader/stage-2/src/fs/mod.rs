use core::{slice::from_raw_parts, mem::zeroed};
//use crate::{println, prints::ToString, vga::get_vga};

use self::{bootsector::BootSector, directory::DirectoryEntry, file::File};

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

/* ==== CONTRUCTOR ========================================================== */
impl FS {
    /// Returns FS instance (Fat12 driver) for the disk associated to the given
    /// disk number. Initial data are read using extern methods that call BIOS
    /// for disk I/O to retrieve physical disk metadata and bootsector data.
    pub fn new(drive_number: u8) -> Self {

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

        //>println!("Number: ", drive_number, " - Type: ", drive_type, " - Cylinders: ", max_cylinders, " - Heads: ", max_heads, " - Sectors: ", max_sectors);
        
        // Initialize Self struct with zeroed boot sector (empty), to be filled.
        let fs: Self = Self {
            drive_number,
            //drive_type, max_cylinders,
            max_heads, max_sectors,

            boot_sector: unsafe { zeroed() },
            fat_buffer: unsafe { zeroed() }, fat_sector: 0, 
            root_buffer: unsafe { zeroed() }, root_sector: 0
        };

        // For the boot sector, read one sector (count=1) at disk start (lba=0).
        // Call BIOS to load boot sector into memory at [&boot_sector].
        // The boot sector is only 1 sector wide (512 bytes), so count = 1.
        // If the BIOS can't read from disk, panic.
        // TODO: before panicking, try resetting and retrying up to 3 times
        let addr: *const u8 = &fs.boot_sector as *const BootSector as *const u8;
        fs.read_disk(0, 1, addr, b"Boot Sector");

        //*! FAT and Root Directory buffers are not initialized, data is read
        //*! from the disk when fat_buffer_read and root_buffer_read are called.

        fs
    }
}


/* ==== FAT ================================================================= */
impl FS {
    /// Reads the FAT entry located at the given index/cluster from the FAT.
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

    /// Reads the byte at the given index from the FAT stored in the buffer.
    /// The buffer doesn't store the whole FAT: if the requested value is out
    /// of range, a disk read is performed to load the correct data first.<br>
    /// TODO: check out of bounds (requested index > maximum FAT size)
    /// TODO:   this shouldn't be necessary since the caller already checks.
    fn fat_buffer_read(&mut self, entry_index: usize) -> Option<&u8> {
        // Initialize min and max entry to the actual FAT entries stored in FS
        // instance. If there is not fat sector loaded, max and min would be 0.
        let max_entry: usize = self.fat_sector * 512;
        let min_entry: usize = max_entry.saturating_sub(512 - 1);

        // If entry is out of range, buffer correct entries before reading.
        // If FAT is not loaded, min and max are 0, so entry + 1 is always out
        // of range and the disk is read.
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
}


/* ==== DIRECTORIES ========================================================= */
impl FS {
    /// Path separator character, also used as first char for root directory.
    const PATH_SEPARATOR: u8 = b'/';

    /// Searches for the entry at the provided path starting from the root
    /// directory and following eventual sub-directories.
    /// - If any of the path values is not found, [None] is returned.
    /// - If a path directory is found but is file instead, [None] is returned.
    pub fn get_entry_from_absolute_path(&mut self, path: &[u8]) -> Option<DirectoryEntry> {

        // Split the string slice at '/'s and convert to iterator.
        // Since this is an absolute path, always ignore first char (/).
        if path[0] != FS::PATH_SEPARATOR { panic!("Path must be absolute"); };
        let mut path = path[1..].split(|char| *char == FS::PATH_SEPARATOR);

        // Parse original entry name to fit Fat12 format
        let root_entry_name: [u8; 11] = Self::parse_entry_name(path.next()?);
        let root_entry_name: &[u8] = root_entry_name.as_slice();
        //>println!("Reading entry: \"", root_entry_name, "\"");

        // Get first entry from root directory
        // If directory entry has not been found, return None ('?')
        let mut entry: DirectoryEntry = self.get_entry_from_root(root_entry_name)?.clone();

        // If directory entry has been found, keep iterating through the path
        loop {
            let entry_name: [u8; 11] = match path.next() {
                // If iterator is consumed, exit loop and return previous entry
                None => break,

                // If there's another element in the path, the previous must be
                // a directory. If not, return None.
                // If the entry name is empty, ignore and go to next loop (//).
                // If it is, parse next name and search it in previous dir.
                Some(e) => {
                    if !entry.is_directory() { return None; }
                    if e.is_empty() { continue; }
                    Self::parse_entry_name(e)
                }
            };


            // Create file instance using the previos directory metadata
            let mut file: File = File::new(entry);

            // Read the entries of the directory and retrieve the one we need.
            // If there's actually no entry with the given name, return None.
            //>println!("Reading entry: \"", entry_name.as_slice(), "\"");
            entry = self.get_entry_from_directory(&mut file, entry_name.as_slice())?.clone();
        }

        // Entry is not a reference (&) because there would be lifetime issues
        // since we are mutably referencing self in a loop and returning a
        // lifetime that would be bound to self from the method.
        Some(entry)
    }

    /// The number of directory entries that can be stored in one disk sector.
    /// Used to calculate entry indexes when reading from root directory.
    const ENTRIES_PER_SECTOR: usize = 512 / 32;

    /// Searches for the entry with the provided name in the root directory.
    /// If the needed entry is not currently loaded to the root directory
    /// buffer, a disk read is performed to load the correct data.
    /// The loaded directory entries are checked against the given entry name.
    /// If an entry that matches it is found, it is returned.
    fn get_entry_from_root(&mut self, name: &[u8]) -> Option<&DirectoryEntry>{

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

    /// The maximum number of directory entries that can be stored in the file
    /// buffer depends on the file buffer size and entry size (32).
    const ENTRIES_PER_FILE_BUFFER: usize = File::BUFFER_SIZE * File::SECTION_SIZE / 32 - 1;

    /// Searches for the entry with the provided name in the given directory.
    /// The given file is reset and read, then the loaded directory entries are
    /// checked against the given entry name. If an entry that matches it is
    /// found, it is returned.<br>
    /// TODO: only take DirectoryEntry (metadata) in input and create file here
    pub fn get_entry_from_directory(&mut self, file: &mut File, entry_name: &[u8]) -> Option<&DirectoryEntry> {
        // Since we need to seek each entry in the directory, we want to start
        // from the first - if for some reason the file has already been read,
        // reset the reading metadata and start from 0.
        if file.current_cluster_read_sectors != 0 { file.reset(); }

        // Read first chunk of file data from disk and place it in the buffer
        self.file_read(file);

        // Cast byte buffer to entries buffer so that we can loop through them
        let dir_buffer: &[DirectoryEntry] =  unsafe { from_raw_parts(&file.buffer as *const u8 as *const DirectoryEntry, File::SECTION_SIZE * File::BUFFER_SIZE / 32) };
        
        // Initialize starting index, max/min values for first chunk of data,
        // which depend on the file buffer size.
        let mut entry_index: usize = 0;
        let mut max_entry: usize = Self::ENTRIES_PER_FILE_BUFFER;
        let mut min_entry: usize = 0;
        loop {

            // If entry is out of range, read next chunk of file data from disk.
            // Since we are moving to the next set of entries, also increment
            // min and max entry index values by entries per buffer size.
            if entry_index < min_entry || max_entry < entry_index {
                //>println!("Min: ", min_entry, " - Max: ", max_entry, " - Index: ", entry_index);
                self.file_read(file);
                max_entry += Self::ENTRIES_PER_FILE_BUFFER;
                min_entry += Self::ENTRIES_PER_FILE_BUFFER;
            }

            // Get relative entry index (0-223 --> 0-15) and read from buffer.
            let entry: &DirectoryEntry =  dir_buffer.get(entry_index % Self::ENTRIES_PER_FILE_BUFFER)?;

            // If name's first byte is NULL, there are no more entries, exit.
            if *entry.name.get(0)? == 0x00 { break; }

            // If the entry name matches the given file name, return this entry.
            if entry_name.eq(&entry.name) { return Some(entry); }

            entry_index += 1;
        }
        None
    }
}


/* ==== FILES =============================================================== */
impl FS {
    /// Searches for the given file starting from the root directory and
    /// returns a File instance with a copy of its metadata.
    /// Searching for the file name could involve reading more root directory
    /// entries than are buffered (if any is buffered), so reading from disk
    /// and updating stored data might happen (hence, the mutable reference).
    /// TODO: implement disk WRITE
    pub fn get_file_from_absolute_path(&mut self, path: &[u8]) -> Option<File> {
        let entry: DirectoryEntry = self.get_entry_from_absolute_path(path)?;
        Some( File::new(entry) )
    }

    /// Fills the File buffer with its actual content read from the disk.
    /// If the File cluster info are present, they're used to load the next
    /// data to the buffer, overriding the previous content.
    /// Reading file content could involve following the FAT for cluster lookup
    /// in non buffered entries (if any is buffered), so reading from disk and
    /// updating stored data might happen (hence, the mutable reference).
    pub fn file_read(&mut self, file: &mut File) -> () {
        // Reset finished property to false, if the file cannot be contained in
        // the buffer, caller could use it and would expect it at false.
        file.finished = false;

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

            // If cluster number is >= FF8, that was the last cluster
            // Reset file read metadata and exit
            if current_cluster >= 0x0FF8 {
                file.current_cluster = file.metadata.lower_first_cluster;
                file.current_cluster_read_sectors = 0;
                file.finished = true;
                break;
            }

            // If the buffer is full, save reading metadata and exit
            if buffer_capacity == 0 {
                file.current_cluster = current_cluster;
                file.current_cluster_read_sectors = current_cluster_read_sectors;
                break;
            }
        }
    }
}


/* ==== I/O ================================================================= */
impl FS {
    /// Translates the LBA (Logical Block Address, the sector we need to read
    /// from disk starting from 0) into CHS (Cylinder Head Sector, physical
    /// coordinates of the sector on the disk), to be used in disk I/Os.
    fn lba_to_chs(lba: u16, max_sectors: u8, max_heads: u8) -> (u16, u8, u8) {
        let max_sectors = max_sectors as u16;
        let max_heads = max_heads as u16;
        (
            (lba / max_sectors) / max_heads,            // C
            ((lba / max_sectors) % max_heads) as u8,    // H
            ((lba % max_sectors) + 1) as u8             // S
        )
    }

    /// Uses disk metadata to load data from disk at given memory location.
    /// It translates the LBA into CHS and uses it to call an extern ASM
    /// method that reverts CPU to real mode, performs disk I/O using BIOS
    /// interrupts to load data to designated memory address and sets protected
    /// mode again.
    /// 
    /// Loading address must be less than the maximum real mode segmented
    /// memory limit, since the address is translated and passed to the BIOS
    /// in real mode.
    fn read_disk(&self, lba: u16, count: u8, addr: *const u8, _reason: &[u8]) {
        let (cylinder, head, sector) = FS::lba_to_chs(lba, self.max_sectors, self.max_heads);

        //>println!("LBA: ", lba, " - CHS: ", cylinder, "/", head, "/", sector, " - Count: ", count, " - Addr: ", addr.to_string_base(16), " < ", _reason);

        let outcome: bool = unsafe { _c_disk_read(self.drive_number, cylinder, head, sector, count, addr) };
        if !outcome { panic!("Could not read from disk!"); }
    }

}


/* ==== PATH AND FILE PARSING =============================================== */
impl FS {
    /// Maximum supported size for FAT12 entry names.
    const ENTRY_NAME_LENGTH: usize = 11;

    /// Parses an ASCII string to be used in the Fat12 I/O operations.
    /// The given string is uppercased, file name and extension are respectively
    /// place at the start and at the end of the 11 byte string, with spaces
    /// in between. If the string is longer than 11, it is truncated.
    /// 
    /// # Examples
    /// ```
    /// assert_eq!(FS::parse_entry_name(b"test.bin"), b"TEST    BIN");
    /// assert_eq!(FS::parse_entry_name(b"dir"), b"DIR        ");
    /// assert_eq!(FS::parse_entry_name(b"iamlongerthan11"), b"IAMLONGERTH");
    /// ```
    pub fn parse_entry_name(mut entry_name: &[u8]) -> [u8; FS::ENTRY_NAME_LENGTH] {

        // If the file name length is > 11, take a slice
        if entry_name.len() > FS::ENTRY_NAME_LENGTH {
            entry_name = &entry_name[..11];
        }

        // Initialize parsed name buffer with empty spaces
        let mut parsed_entry_name: [u8; FS::ENTRY_NAME_LENGTH] = [b' '; FS::ENTRY_NAME_LENGTH];

        // Initialize extension start position to file length (no extension)
        let mut extension_index: usize = entry_name.len();

        // Enumerate through file name characters from the end;
        // if a '.' is found, that is the start of the file extension.
        for (i, char) in entry_name.iter().enumerate().rev() {
            if *char == b'.' {
                extension_index = i;
                break;
            }
        }

        // Parse name before the extension (from 0 to extension index).
        // Place the uppercase name starting from the start of the buffer.
        for i in 0..extension_index {
            parsed_entry_name[i] = entry_name[i].to_ascii_uppercase();
        }

        // Parse extension (from extension index to file name length).
        // Place the uppercase extension starting from the end of the buffer.
        // Characters between name and extension are set to empty space.
        for i in (extension_index+1..entry_name.len()).rev() {
            parsed_entry_name[FS::ENTRY_NAME_LENGTH-(entry_name.len()-i)] = entry_name[i].to_ascii_uppercase();
        }

        // Return parsed file name buffer ("test.bin" --> "TEST    BIN")
        parsed_entry_name
    }
}