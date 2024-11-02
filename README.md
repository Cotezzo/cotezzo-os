# Cotezzo-OS
## Introducton
Simple and useless operating system developed following guides and documentations.
Not aiming to turn this into a complete and functional operating system, but trying to implement as many functionalities as possible for the sake of learning.

## Structure
As of now, the project only supports BIOS; has a stage-1 bootloader written in assembly (nasm), a stage-2 bootloader and a kernel both written in Rust.

The stage-1 works as intended: it loads from the FAT12 fs the stage-2 into memory and executes it.
The stage-2 is correctly invoked and loads into memory the actual kernel binary after switching to 32 bit protected mode. The stage-2 is not yet collecting enough system informations for the kernel to use.

Both the stage-2 and the kernel are freestanding Rust binaries compiled with a custom triple for 32bit architecture, and use some assembly methods linked with a custom linker script.
Building the project could take a while since recompiling the standard library from source code may be required.
The final binary is stripped from the ELF output file and is put in the disk image as a file.

NOTE: Rust modules involve assembly and could be unpredictable in dev mode, using the release target is recommended.

## Usage
### Installation
Assert that your machine supports all the commands used in the Makefile (or change them).
The main packages (apt) used are:
- `nasm`: assembler for our x86 ISA
- `qemu-system`: emulation software
- `dosfstools` and `mtools`: filesystem utils
- `cargo`: package manager, builder, compiler for Rust (currently using cargo 1.76.0-nightly)

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

When using VSCode's terminal, some errors might occur during compilation.
To avoid these errors, the GTK_PATH env variable must be unset using the command `unset GTK_PATH`.

All the assembler, emulation and debugging configurations are set in the Makefile.
The output directory for binary and image files is the `target` directory.

## Theory
### TODO: summarize and organize key concepts
- BIOS, Bootloader and Kernel
- Memory Segmentation
- Stack
- Interrupts
- CHS and LBA addressing schemes
- FAT file system
- Real Mode vs Protected Mode
- Port Mapped I/O
- Text Mode VGA
- GDT and IDT