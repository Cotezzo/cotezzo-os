/**! Linker Script passed to the compiler, configured in .cargo/config.toml.
  *! Not really useful for now, since the binary is stripped from the ELF.    */

/* Specify executable entry point (first section) */
ENTRY(_start)

SECTIONS{
    /* Place the .text section at the start of the output binary. */
    . = 0x0;
    .text : { *(.text*) }

    /* Remove ELF metadata from the output file. */
    /DISCARD/ : { *(.comment) }
}