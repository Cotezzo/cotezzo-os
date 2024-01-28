use core::mem::zeroed;

use super::directory::DirectoryEntry;

/* ==== TYPE DEFINITION ===================================================== */
pub struct File<'fs> {
    pub metadata: &'fs DirectoryEntry,
    pub current_cluster: u16,
    pub current_cluster_read_sectors: u16,
    pub buffer: [u8; File::SECTION_SIZE * File::BUFFER_SIZE]
}

/* ==== TYPE CONSTANTS ====================================================== */
impl<'fs> File<'fs> {
    /** Size of the minimum readable memory from the disk. */
    pub const SECTION_SIZE: usize = 512;
    /** Size of file buffer in sections. */
    pub const BUFFER_SIZE: usize = 2;
}

/* ==== STATIC TYPE METHODS ================================================= */
impl<'fs> File<'fs> {
    /** Creates a File instance, which contains metadata and reading state.
        The file content buffer is also initialized, but it's empty.
        In order to fill it, use the file_read method of a Fat12 instance. */
    pub fn new(metadata: &'fs DirectoryEntry) -> Self {
        Self {
            metadata,
            current_cluster: metadata.lower_first_cluster,
            current_cluster_read_sectors: 0,
            buffer: unsafe { zeroed() }
        }
    }
}