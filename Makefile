# Define source and build directory where the .asm and output files are stored.
SRC_DIR_PATH=./src
BUILD_DIR_PATH=./build

# Define source assembly code files and binary outputs.
BOOTLOADER_PATH=${SRC_DIR_PATH}/bootloader/bootloader.asm
BOOTLOADER_BIN_PATH=${BUILD_DIR_PATH}/bootloader.bin
KERNEL_PATH=${SRC_DIR_PATH}/kernel/kernel.asm
KERNEL_BIN_NAME=kernel.bin
KERNEL_BIN_PATH=${BUILD_DIR_PATH}/${KERNEL_BIN_NAME}

# Define image file, block size and block count (disk dimensions)
# Our "floppy" has a 1.44MB of memory
IMAGE_PATH=${BUILD_DIR_PATH}/floppy.img
BLOCK_SIZE=512
BLOCK_COUNT=2880	# ~1.47MB

# Define assembler command and options.
ASM=nasm
ASM_OPTIONS=-f bin -o

# Define QEMU command; -fda is used to load .img file as a disk.
# Intel Arch, 32bit (aka i386). ISA: x86-32, 32b version of x86 (16b).
EMU=qemu-system-i386 -fda ${IMAGE_PATH}

# Define .gdb debug script file and content (\ \n for multiline support).
GDB_SCRIPT_PATH=${BUILD_DIR_PATH}/debug-script.gdb
GDB_CONFIG="\
\nset disassembly-flavor intel\
\ntarget remote | ${EMU} -S -gdb stdio -m 32\
\nlayout asm"

# ==== ALL =================================================================== #
# Default behaviour: create build dir, create floppy image from source.
all: ${BUILD_DIR_PATH} ${IMAGE_PATH}
${BUILD_DIR_PATH}:
	mkdir -p ${BUILD_DIR_PATH}

# ==== IMAGE ================================================================= #
# Create the .img file from bootloader and kernel binaries.
# 'dd' performes file operations. Create an output file (of) from input file
# (if) data: /dev/zero is a virtual file which only outputs 0x00 (NULL);
# it is useful to create empty files and allocate space. 'bs' defines the block
# size (sectors size), count defines how many blocks - that gives the disk size.
# 'mkfs.fat' "formats" the given device/file to FAT fs (-F 12 -> FAT12).
# The second 'dd' command writes at the start of the image our bootloader;
# notrunc tells the command not to remove all the rest of the existing file.
# Copy the kernel binary to the root directory of the FAT12 image, this can
# be done without mounting the image using the mtools commands (such as mcopy).
#! This would overwrite the FAT12 headers and break the file system!
#! The bootloader itself contains a copy of these headers to keep the fs valid.
${IMAGE_PATH}: ${BOOTLOADER_BIN_PATH} ${KERNEL_BIN_PATH}
	dd if=/dev/zero of=${IMAGE_PATH} bs=${BLOCK_SIZE} count=${BLOCK_COUNT}
	mkfs.fat -F 12 ${IMAGE_PATH} -n "MY_NAME"
	dd if=${BOOTLOADER_BIN_PATH} of=${IMAGE_PATH} conv=notrunc
	mcopy -i ${IMAGE_PATH} ${KERNEL_BIN_PATH} "::${KERNEL_BIN_NAME}"

# ==== BOOTLOADER ============================================================ #
# Create bootloader binary from assembly source.
${BOOTLOADER_BIN_PATH}: ${BOOTLOADER_PATH}
	${ASM} ${BOOTLOADER_PATH} ${ASM_OPTIONS} ${BOOTLOADER_BIN_PATH}

# ==== KERNEL ================================================================ #
# Create kernel binary from assembly source.
${KERNEL_BIN_PATH}: ${KERNEL_PATH}
	${ASM} ${KERNEL_PATH} ${ASM_OPTIONS} ${KERNEL_BIN_PATH}

# ==== RUN =================================================================== #
# Build and run os with the defined QEMU command and options.
run: all
	${EMU}

# Build and debug os with QEMU and GDB; load .gdb config and scripts.
# Useful debugging commands:
# SET BREAKPOINT:                               b *0x7c00
# VIEW REGISTERS VALUE:                         i r ax bx cx dx si di pc sp
# VIEW SOURCE ALONGSIDE DISASSEMBLY:            layout split
# VIEW DISASSEMBLY ONLY:                        layout asm
# VIEW RAM (ex: STACK, 16*h(2B) from $sp addr): x/16xh $sp
dbg: all
	echo ${GDB_CONFIG} > ${GDB_SCRIPT_PATH}
	gdb -tui -x ${GDB_SCRIPT_PATH}

# ==== CLEAN ================================================================= #
clean:
	rm ${BUILD_DIR_PATH}/*