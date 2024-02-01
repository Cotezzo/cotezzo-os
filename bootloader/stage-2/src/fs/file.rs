use core::mem::zeroed;

use super::directory::DirectoryEntry;

/* ==== TYPE DEFINITION ===================================================== */
pub struct File {
    /** Copy of the DirectoryEntry associated with the file in the FileSystem */
    pub metadata: DirectoryEntry,

    /** Current cluster stored in the buffer */
    pub current_cluster: u16,
    /** Buffered sectors of the current cluster stored in the buffer */
    pub current_cluster_read_sectors: u16,
    /** Buffer used to store the content of the file during read operations */
    pub buffer: [u8; File::SECTION_SIZE * File::BUFFER_SIZE]
}

/* ==== TYPE CONSTANTS ====================================================== */
impl File {
    /** Size of the minimum readable memory from the disk. */
    pub const SECTION_SIZE: usize = 512;
    /** Size of file buffer in sections. */
    pub const BUFFER_SIZE: usize = 1;
}

/* ==== STATIC TYPE METHODS ================================================= */
impl File {
    /** Creates a File instance, which contains metadata and reading state.
        The file content buffer is also initialized, but it's empty.
        In order to fill it, use the file_read method of a Fat12 instance. */
    pub fn new(metadata: DirectoryEntry) -> Self {
        Self {
            current_cluster: metadata.lower_first_cluster,
            current_cluster_read_sectors: 0,
            buffer: unsafe { zeroed() },
            metadata
        }
    }
}