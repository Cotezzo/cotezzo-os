/**! Linker Script passed to the compiler, configured in <custom-triple>.json
 * //.cargo/config.toml.  */

/*  Specify executable entry point (_start method, under .text.start section) */
ENTRY(_start)

SECTIONS {
    /*  Stage-1 loads the code at 0x0:0x0500, so use this offset value for the
        address calculations. */
    . = 0x500;

    /*  Create a .text.start section at the very start of the final ELF binary,
        it will contain the .text.start section declared in main.asm and all
        its subsections (hence, .text.start*).
        This section is then followed by .text with all the other code.
        
        Output ELF sections could be declared with a different name.
        Ex: `.asd : { *(.text*) }` All code under .text* will be placed in .asd.
        */
    .text.start :       { *(.text.start*)       }
    .text :             { *(.text*)             }

    /*  Data, Read-Only data sections - Separate GDT because we can (useless) */
    .data :             { *(.data*)             }
    .rodata.gdt :       { *(.rodata.gdt*)       }
    .rodata :           { *(.rodata*)           }

    /*  Block Started by Symbol section - For uninitialized global variables.
        Variables in this section are usually zero-initialized by the system. */
    .bss :              { *(.bss*)              }

    /*  Global Offset Table section - Contains dynamic linking data for runtime
        addresses resolution when using dynamic libraries. */
    .got :              { *(.got*)              }

    /*  Exception Handling section - Contains data used for stack unwinding.
        HDR section provides binary search table for unwinding optimization. */
    .eh_frame :         { *(.eh_frame*)         }
    .eh_frame_hdr :     { *(.eh_frame_hdr*)     }

    /*  Remove ELF metadata from the output file. */
    /DISCARD/ :         { *(.comment*)          }

    /*  Store and declare the address of the binary's last byte.
        This value can be used by importing it as extern. */
    end = .;
}