/**! Linker Script passed to the compiler, configured in .cargo/config.toml.
  *! Not really useful for now, since the binary is stripped from the ELF.    */

/* Specify executable entry point (first section) */
ENTRY(_start)

SECTIONS{
    /*  Place the .text._start section at the very start of the output binary,
        followed by all the other .text sections. */
    . = 0x0;
    .text : {
      *(.text._start)
      *(.text*)
    }

    /* Force the .rodata section after the start section. */
    .rodata : {
        *(.rodata .rodata.*)
    }

    /* Remove ELF metadata from the output file. */
    /DISCARD/ : { *(.comment) }
}