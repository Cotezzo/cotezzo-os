/* ==== TYPE DEFINITION ===================================================== */
/*  Define FAT12 headers and bootloader sector.
    All the header values are mapped, but the bootloader code is ignored. */
#[repr(C, packed)]
pub struct BootSector {
    // BIOS Parameter Block
    jump_instruction: [u8; 3],
    oem_id: [u8; 8],
    bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_count: u8,
    pub root_entries : u16,
    sector_count: u16,
    media_descriptor: u8,
    pub sectors_per_fat: u16,
    sectors_per_cylinder: u16,
    heads_count: u16,
    hidden_sectors_count: u32,
    large_sector_count: u32,

    // Extended Boot Record
    drive_number: u8,
    reserved: u8,
    volume_id : u32,
    volume_label : [u8; 11],
    system_id: [u8; 8],

    /*  BootLoader code (ignored)
    !   We need this padding or we'll overflow into memory, disk reading
    !   only works in chunks of 512 byte. */
    padding: [u8; 512 - 61]
}

/* ==== TYPE METHODS ======================================================== */
impl BootSector {
    /** Returns the sector on the disk in which the FAT starts. */
    pub fn get_fat_offset(&self) -> u16 { self.reserved_sectors }
    /** Returns the size in sectors of a single FAT. */
    pub fn get_fat_size(&self) -> u16 { self.sectors_per_fat }

    /** Returns the sector on the disk in which the root directory starts.
        The root directory is placed right after the FATs. */
    pub fn get_root_offset(&self) -> u16 { self.get_fat_offset() + (self.get_fat_size() * self.fat_count as u16) }
    /** Returns the size in sectors of the full root directory.
        Complete formula would be: entries * entry_size / sector_size */
    pub fn get_root_size(&self) -> u16 { self.root_entries / 16 }

    /** Returns the sector on the disk in which the data cluster start.
        The clusters are placed right after the root directory. */
    pub fn get_cluster_region_offset(&self) -> u16 { self.get_root_offset() + self.get_root_size() }
    /** Returuns the secton on the disk in which the given cluster starts.
        The given cluster number has to account for the empty FAT entries. */
    pub fn get_cluster_offset(&self, cluster: u16) -> u16 {
        self.get_cluster_region_offset() + (self.get_cluster_size() * (cluster - 2))
    }
    /** Returns the size in sectors of a single cluster. */
    pub fn get_cluster_size(&self) -> u16 { self.sectors_per_cluster as u16}
}