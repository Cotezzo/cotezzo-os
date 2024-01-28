/* ==== TYPE DEFINITION ===================================================== */
#[repr(C, packed)]
pub struct DirectoryEntry {
    pub name: [u8; 11],
    pub attributes: u8,             // READ_ONLY=0x01 HIDDEN=0x02 SYSTEM=0x04 VOLUME_ID=0x08 DIRECTORY=0x10 ARCHIVE=0x20 LFN=READ_ONLY|HIDDEN|SYSTEM|VOLUME_ID (LFN means that this entry is a long file name entry)
    reserved: u8,
    pub creation_time_tenths: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_access_date: u16,
    pub upper_first_cluster: u16,
    pub last_change_time: u16,
    pub last_change_date: u16,
    pub lower_first_cluster: u16,
    pub file_size: u32
}   // 32 byte


/* ==== TYPE METHODS ======================================================== */
impl DirectoryEntry {
    /* Gets the u32 cluster number of the described file. *
    pub fn get_cluster(&self) -> u32 {
        ((self.upper_first_cluster as u32) << 16) | self.lower_first_cluster as u32
    } */
}