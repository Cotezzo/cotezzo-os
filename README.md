# Porcheria-OS
## Introducton
Simple and useless operating system developed following guides and documentations.
Not aiming to turn this in a complete and functional operating system, but trying to implement as many functionalities as possible for the sake of learning.

## Structure
As of now, the project only supports BIOS; has a stage-1 bootloader written in assembly (nasm), a stage-2 bootloader written in Rust (for fun), and a kernel.
The stage-1 is working as intended: it loads from the FAT12 fs the stage-2 into memory and executes it.
The stage-2 is correctly invoked but not yet implemented: it should collect data from the bios and prepare the environment for the kernel.
The kernel is not yet loaded into memory nor implemented (the current file is just a placeholder).

The stage-2 is a freestanding Rust binary compiled with a custom triple for our 16bit architecture, and uses some assembly methods linked with a custom linker script.
Building the project could take a while since recompiling the standard library from source code is required.
The final binary is stripped from the ELF output file and is put in the disk image as a file.

NOTE: stage-2 involves assembly and is very unpredictable in dev mode, using the release target is recommended.

## Usage
### Installation
Assert that your machine supports all the commands used in the Makefile (or change them).
The main packages (apt) used are:
- `nasm`: assembler for our x86 ISA
- `qemu-system`: emulation software
- `dosfstools` and `mtools`: filesystem utils
- `cargo`: package manager, builder, compiler for Rust

### Other useful tools
- `GHex` (or any other hex editor): useful to study the "anatomy" of the finished image file and assert that the file system and files are in the expected state and location.
- `gdb`: debugging software that can be used to watch the state of run instructions, CPU registers and memory step by step, in order to better understand machine code execution and find bugs.

### Makefile
The root Makefile invokes all the Makefiles from the sub-projects
(currently stage-1, stage-2, kernel) and creates a bootable disk image.

To create a clean project build and run the output disk with qemu, using
`make clean run TARGET=release` is recommended. Other supported targets are:
- `make`, `make all` or  `make TARGET=all`: build the image with default target.
- `make TARGET=dev`: build the image using the dev target for all the sub-projects.
- `make TARGET=release`: build the image using the release target for all the sub-projects.
- `make run`: build the image and run QEMU.
- `make dbg`: build the image and run QEMU + GDB debugger.
- `make clean`: deletes the project and sub-projects build directories. Necessary before a
new build if linker scripts or assembly files used in the Rust modules have been modified,
since cargo can't watch those file for us.

All the assembler, emulation and debugging configurations are set in the Makefile.
The output directory for binary and image files is the `target` directory.

## Theory
### TODO: summarize and organize key concepts
- BIOS, Bootloader and Kernel
- Memory Segmentation
- Stack
- Signal Interrupts
- CHS and LBA addressing schemes
- FAT file system

The source code is well commented anyway.