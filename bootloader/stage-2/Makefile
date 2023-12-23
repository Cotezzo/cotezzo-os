# Define project name and custom triple (target) file for the compiler
# The custom triple is defined as a .json file stored in repository root
PROJECT_NAME=stage-2
CUSTOM_TRIPLE=i386-code16-stage-2

# Define source Rust code location
SRC_RS=./src/main.rs
SRC_ASM=./src/print.asm

# Define target folder and output ELFs location
TARGET_DIR=./target
TARGET_ASM_ELF=${TARGET_DIR}/asm.o
TARGET_DEBUG_DIR=${TARGET_DIR}/${CUSTOM_TRIPLE}/debug
TARGET_RELEASE_DIR=${TARGET_DIR}/${CUSTOM_TRIPLE}/release
TARGET_DEBUG_ELF=${TARGET_DEBUG_DIR}/${PROJECT_NAME}
TARGET_RELEASE_ELF=${TARGET_RELEASE_DIR}/${PROJECT_NAME}

# Define output binary location
TARGET_BIN=${TARGET_DIR}/${PROJECT_NAME}.bin

# ==== TARGET ================================================================ #
# Don't treat these targets as files
.PHONY: all dev release clean bin elf

# DEFAULT: build project if source has changed
all: dev

# objcopy is enough to create the final binary, but it discards compiler errors.
#! NOTE: more build & compile options are specified in .cargo/config.toml.
# DEV: recompile and extract binary
dev: ${TARGET_DEBUG_ELF}
${TARGET_DEBUG_ELF}: ${SRC_RS} ${TARGET_DIR} ${TARGET_ASM_ELF}
	cargo build
	cargo objcopy -- -O binary ${TARGET_BIN}

# RELEASE: recompile with optimization and extract binary
release: ${TARGET_RELEASE_ELF}
${TARGET_RELEASE_ELF}: ${SRC_RS} ${TARGET_DIR} ${TARGET_ASM_ELF}
	cargo build --release
	cargo objcopy --release -- -O binary ${TARGET_BIN}

# Create target directory for the bin
${TARGET_DIR}:
	mkdir -p ${TARGET_DIR}

# Create asm binary from assembly source - it'll be linked with the rust module
${TARGET_ASM_ELF}: ${SRC_ASM}
	nasm ${SRC_ASM} -f elf32 -o ${TARGET_ASM_ELF}

# Delete target and binary files
clean:
	rm -rf ${TARGET_DIR}

# ==== UTILITY & DEBUG ======================================================= #
# Disassemble of the raw binary content (i386 ISA, Intel syntax)
bin: ${TARGET_RELEASE_ELF} ${TARGET_BIN}
	objdump ${TARGET_BIN} -D -b binary -m i386 -M intel | less

# Disassemble content, read .comment ELF generated metadata section (useless)
elf: ${TARGET_RELEASE_ELF}
	objdump ${TARGET_RELEASE_ELF} -D -m i386 -M intel | less
	readelf -p .comment ${TARGET_RELEASE_ELF}