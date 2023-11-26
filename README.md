# Porcheria-OS
## Introducton
Simple and useless operating system developed following guides and documentations.
Not aiming to turn this in a complete and functional operating system, but trying to implement as many functionalities as possible for the sake of learning.

There may be different branches with experimental implementations.

## Usage
### Installation
Assert that your machine supports all the commands used in the Makefile (or change them).
The main packages (apt) used are:
- `nasm`: assembler for our x86 ISA
- `qemu-system`: emulation software
- `dosfstools` and `mtools`: filesystem utils

### Other useful tools
- `GHex` (or any other hex editor): useful to study the "anatomy" of the finished image file and assert that the file system and files are in the expected state and location.
- `gdb`: debugging software that can be used to watch the state of run instructions, CPU registers and memory step by step, in order to better understand machine code execution and find bugs.

### Makefile
- `make`: build the image.
- `make run`: build the image and run QEMU.
- `make dbg`: build the image and run QEMU + GDB debugger.

All the assembler, emulation and debugging configurations are set in the Makefile.
The output directory for binary and image files is the `build` directory.

## Theory
### TODO: summarize and organize key concepts
- BIOS, Bootloader and Kernel
- Memory Segmentation
- Stack
- Signal Interrupts
- CHS and LBA addressing schemes
- FAT file system

The source code is well commented anyway.