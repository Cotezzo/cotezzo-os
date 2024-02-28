/**! Linker Script passed to the compiler, configured in <custom-triple>.json */

/*  Specify executable entry point */
ENTRY(_rs_start)

SECTIONS {
    /*  Stage-2 loads kernel at 0x10000 */
    . = 0x10000;

    /*  Specify _rs_start as actual first entry or it may not work...
        Don't know what ENTRY(_rs_start) is used if it doesn't work. */
    .text :             { *(.text._rs_start*)   }
    .text :             { *(.text*)             }
    .data :             { *(.data*)             }
    .rodata :           { *(.rodata*)           }
    .got :              { *(.got*)              }
    .eh_frame :         { *(.eh_frame*)         }
    .eh_frame_hdr :     { *(.eh_frame_hdr*)     }

    /DISCARD/ :         { *(.comment*)          }

    end = .;
}