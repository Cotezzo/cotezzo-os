# Input target, can be 'all', 'dev', 'release' (ex: make TARGET=release)
TARGET?=all

# Define bootloader stages target binary location
STAGE_1_DIR=./bootloader/stage-1
STAGE_2_DIR=./bootloader/stage-2
KERNEL_DIR=./kernel
TARGET_STAGE_1_BIN=${STAGE_1_DIR}/target/stage-1.bin
TARGET_STAGE_2_BIN=${STAGE_2_DIR}/target/stage-2.bin
TARGET_KERNEL_BIN=${KERNEL_DIR}/target/kernel.bin

# Define output image location
TARGET_DIR=./target
TARGET_IMG=${TARGET_DIR}/porcheria-os.img

# Define image file, block size and block count (disk dimensions)
# Our "floppy" has a 1.44MB of memory
BLOCK_SIZE=512
BLOCK_COUNT=2880	# ~1.47MB

# Define QEMU command; -fda is used to load .img file as a disk.
# Intel Arch, 32bit (aka i386). ISA: x86-32, 32b version of x86 (16b).
EMU=qemu-system-i386 -fda ${TARGET_IMG} -d int,cpu_reset -no-reboot -D ${TARGET_DIR}/floppy.log # -nographic > output.txt

# Define .gdb debug script file and content (\ \n for multiline support).
GDB_SCRIPT_PATH=${TARGET_DIR}/debug-script.gdb
GDB_CONFIG="\
\nset disassembly-flavor intel\
\ntarget remote | ${EMU} -S -gdb stdio -m 32\
\nlayout asm\
\nb *0x7c00\
\nc\
\nx/16xh 0x7dfe"

# ==== TARGET ================================================================ #
# Don't treat these targets as files
#! If "kernel" is not in PHONY, the target won't run because the dependancy is
#! already satisfied: there actually is a directory called "kernel".
.PHONY: all clean run dbg stage-1 stage-2 kernel

# DEFAULT: always clean and create new target image
all: ${TARGET_IMG}

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
${TARGET_IMG}: ${TARGET_DIR} stage-1 stage-2 kernel
	dd if=/dev/zero of=${TARGET_IMG} bs=${BLOCK_SIZE} count=${BLOCK_COUNT}
	mkfs.fat -F 12 ${TARGET_IMG} -n "PORK_OS"
	dd if=${TARGET_STAGE_1_BIN} of=${TARGET_IMG} conv=notrunc
	mcopy -i ${TARGET_IMG} ${TARGET_STAGE_2_BIN} "::stage-2.bin"
	mmd -i ${TARGET_IMG} "::kernel"
	mcopy -i ${TARGET_IMG} ${TARGET_KERNEL_BIN} "::kernel/main.bin"

# Create stage-1 bootloader binary from assembly source.
stage-1: #${TARGET_STAGE_1_BIN}:
	make ${TARGET} -C ${STAGE_1_DIR}

# Create stage-2 bootloader binary from assembly source.
stage-2: #${TARGET_STAGE_2_BIN}:
	make ${TARGET} -C ${STAGE_2_DIR}

# Create kernel binary from assembly source.
kernel:
	make ${TARGET} -C ${KERNEL_DIR}

# Create target directory for the image
${TARGET_DIR}:
	mkdir -p ${TARGET_DIR}

# Create target directory for the bin
clean:
	rm -rf ${TARGET_DIR}
	make clean -C ${STAGE_1_DIR}
	make clean -C ${STAGE_2_DIR}
	make clean -C ${KERNEL_DIR}

# ==== RUN =================================================================== #
# Build and run os with the defined QEMU command and options.
run: all
	${EMU}

# Build and debug os with QEMU and GDB; load .gdb config and scripts.
dbg: all
	echo ${GDB_CONFIG} > ${GDB_SCRIPT_PATH}
	gdb -tui -x ${GDB_SCRIPT_PATH}

# Useful debugging commands:
# SET BREAKPOINT:                               b *0x7c00
# VIEW REGISTERS VALUE:                         i r ax bx cx dx si di pc sp
# VIEW SOURCE ALONGSIDE DISASSEMBLY:            layout split
# VIEW DISASSEMBLY ONLY:                        layout asm
# VIEW RAM (ex: STACK, 16*h(2B) from $sp addr): x/16xh $sp