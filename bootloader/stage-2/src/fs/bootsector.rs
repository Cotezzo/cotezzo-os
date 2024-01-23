/* ==== STRUCT DEFINITION =================================================== */
/*  Define FAT12 headers and bootloader sector.
    All the header values are mapped, but the bootloader code is ignored. */
#[repr(C, packed)]
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
    system_id: [u8; 8],

    /*  BootLoader code (ignored)
    !   We need this padding or we'll overflow into memory, disk reading
    !   only works in chunks of 512 byte. */
    padding: [u8; 512 - 61]
}

/*
TODO: try this again
pub union BootSectorUnion {
    pub b: BootSector,
    pub align: [u8; 512]
}
*/

/* ==== STRUCT DEFINITION =================================================== */
impl BootSector {
    /** Returns the sector from which the first FAT starts. */
    pub fn get_fat_start(&self) -> u16 { self.reserved_sectors }

    /** Returns FAT total size in sectors.
        This is the size of a single FAT, not the sum of the total FATs. */
    pub fn get_fat_size(&self) -> u16 { self.sectors_per_fat }

    /*
    pub fn get_root_start(&self) -> u16 { self.get_fat_start() + (self.get_fat_size() * self.fat_count as u16) }
    pub fn get_root_size(&self) -> usize { self.root_entries as usize * size_of::<DirectoryEntry>() }
    pub fn get_cluster_region_start(&self) -> usize { self.get_root_start() as usize + self.get_root_size() }
    pub fn get_cluster_size(&self) -> usize { self.sectors_per_cluster as usize * self.bytes_per_sector as usize }
    pub fn get_cluster_start(&self, cluster: u16) -> usize {
        self.get_cluster_region_start() + (self.get_cluster_size() * (cluster - 2) as usize)
    }
    */
}