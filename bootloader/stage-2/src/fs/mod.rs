pub mod bootsector;

use core::mem::size_of;
use crate::{println, print::{ToStringBase, ToString}, fs::bootsector::BootSector};
use crate::get_vga;

/* ==== ASM EXTERN METHODS ================================================== */
extern "C" {
    pub fn _c_disk_reset(drive: u8) -> bool;
    pub fn _c_disk_read(drive: u8, cylinder: u16, head: u8, sector: u8, count: u8, addr: *const u8) -> bool;
    pub fn _c_disk_get_params(drive: u8, drive_type_out: *const u8, cylinders_out: *const u16, heads_out: *const u8, sectors_out: *const u8) -> bool;
}

/* ==== STRUCT DEFINITION =================================================== */
/* Public struct used from the main module */
pub struct FS {
    pub drive_type: u8,
    pub max_cylinders: u16,
    pub max_heads: u8,
    pub max_sectors: u8,
    pub boot_sector: BootSector
}

/* ==== STATIC METHODS ====================================================== */
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
}

/* ==== STRUCT IMPLEMENTATIONS ============================================== */
impl FS {
    pub fn new(drive_number: u8) -> FS {

        /* ==== DISK PARAMETERS ============================================= */
        // Initialize variables and get disk parameters:
        // _c_disk_get_params is implemented in ASM, it switches to 16rm,
        // calls BIOS INT 13,8 to retrieve disk data and returns to 32pm.
        // If the BIOS can't retrieve the data, return None.
        let drive_type_out: u8 = 0;
        let cylinders_out: u16 = 0;
        let heads_out: u8 = 0;
        let sectors_out: u8 = 0;
        let outcome: bool = unsafe { _c_disk_get_params(drive_number, &drive_type_out, &cylinders_out, &heads_out, &sectors_out) };
        if !outcome { panic!("Could not read disk parameters!"); }

        println!((&drive_type_out as *const u8).to_string_base(16), ": ", drive_type_out, " (Type)");
        println!((&cylinders_out as *const u16).to_string_base(16), ": ", cylinders_out, " (Cylinders)");
        println!((&heads_out as *const u8).to_string_base(16), ": ", heads_out, " (Heads)");
        println!((&sectors_out as *const u8).to_string_base(16), ": ", sectors_out, " (Sectors)");
        
        /* ==== BOOT SECTOR ================================================= */
        // For the boot sector, read one sector (count=1) at disk start (lba=0)
        let lba: u16 = 0;
        let count: u8 = 1;

        // Boot sector is at first disk partition: LBA is 0.
        // Translate LBA to CHS for BIOS call after getting disk parameters.
        let (cylinder, head, sector) = FS::lba_to_chs(lba, sectors_out, heads_out);

        // Initialize boot sector struct zeroed (empty).
        let boot_sector: BootSector = unsafe { core::mem::zeroed() };
        let addr: *const u8 = &boot_sector as *const BootSector as *const u8;

        //> Debug logs
        println!("CHS: ", cylinder, " - ", head, " - ", sector, "\r\nCount: ", count, "\r\nAddr: ", addr.to_string_base(16));

        // Call BIOS to load boot sector into memory at [&boot_sector].
        // The boot sector is only 1 sector wide (512 bytes), so count = 1.
        // If the BIOS can't read from disk, return None.
        // TODO: before panicking, try resetting and retrying up to 3 times
        let outcome: bool = unsafe { _c_disk_read(drive_number as u8, cylinder, head, sector, count, addr) };
        if !outcome { panic!("Could not read boot sector!"); }

        //> Debug logs
        let ptr = unsafe { core::slice::from_raw_parts(addr, 512) };
        println!("Buffer (", ptr.len()," / ", size_of::<BootSector>(), " bytes) value:\r\n", ptr);

        /* ==== FAT ========================================================= */
        // Get FAT start and size on the disk, translate LBA to CHS
        // For now, FAT buffer is hardcoded to handle 9 sectors (9*512 byte)
        let lba = boot_sector.get_fat_start();
        let count = boot_sector.get_fat_size() as u8;
        let (cylinder, head, sector) = FS::lba_to_chs(lba, sectors_out, heads_out);

        // Call BIOS to load FAT into memory at [&fat].
        // If the BIOS can't read from disk, return None.
        //*let fat: Fat = unsafe { core::mem::zeroed() };
        let fat: [u8; 0] = [0; 0];  // TODO
        let addr: *const u8 = &fat as *const u8;

        //> Debug logs
        println!("CHS: ", cylinder, " - ", head, " - ", sector, "\r\nCount: ", count, "\r\nAddr: ", (addr as usize + fat.len()).to_string_base(16));

        //let outcome: bool = unsafe { _c_disk_read(drive_number as u8, cylinder, head, sector, count, addr) };
        if !outcome { panic!("Could not read FAT!"); }



        /* ==== ROOT DIRECTORY ==== *
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
        */


        // Return FS instance ownership
        FS {
            drive_type: drive_type_out,
            max_cylinders: cylinders_out,
            max_heads: heads_out,
            max_sectors: sectors_out,
            boot_sector
        }
    }
}