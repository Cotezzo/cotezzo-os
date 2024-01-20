/**! Linker Script passed to the compiler, configured in <custom-triple>.json
 * //.cargo/config.toml.  */

/*  Specify executable entry point (_start method, under .text.start section) */
ENTRY(_start)

SECTIONS{
    /*  Stage-1 loads the code at 0x0:0x0500, so use this offset value for the
        address calculations. */
    . = 0x500;

    /*  Place the _start (ASM) section at the very start of the output binary,
        followed by _rs_start (Rust) and all other .text and .data sections. */
    .text : {
      *(.text.start)
      *(.text.rs_start)
      *(.text*)
    }

    .data :         { *(.data)                          }

    /* Read-only data section */
    .rodata :       { *(.rodata .rodata.*)              }

    /* Force the .eh_frame_hdr and .eh_frame section after the start section. */
    .eh_frame :     { *(.eh_frame .eh_frame.*)          }
    .eh_frame_hdr : { *(.eh_frame_hdr .eh_frame_hdr.*)  }

    /* Remove ELF metadata from the output file. */
    /DISCARD/ :     { *(.comment)                       }
}