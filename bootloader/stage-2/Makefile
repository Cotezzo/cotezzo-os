# Define project name and custom triple (target) file for the compiler
# The custom triple is defined as a .json file stored in repository root
PROJECT_NAME=stage-2
CUSTOM_TRIPLE=i386-code16-stage-2

# Define source Rust code location
MAIN=./src/main.rs

# Define target folder and output ELFs location
TARGET_DIR=./target
TARGET_DEBUG_ELF=${TARGET_DIR}/${CUSTOM_TRIPLE}/debug/${PROJECT_NAME}
TARGET_RELEASE_ELF=${TARGET_DIR}/${CUSTOM_TRIPLE}/release/${PROJECT_NAME}

# Define output binary location
TARGET_BIN=${TARGET_DIR}/${PROJECT_NAME}.bin

# ==== TARGET ================================================================ #
# DEFAULT: build project if source has changed
all: ${TARGET_DEBUG_ELF}

# DEV: always clean, recompile and extract binary
dev: clean ${TARGET_DEBUG_ELF}
${TARGET_DEBUG_ELF}: ${MAIN} ${TARGET_DIR}
	cargo objcopy --target=${CUSTOM_TRIPLE}.json -- -O binary ${TARGET_BIN}

# RELEASE: always clean, recompile with optimization and extract binary
release: clean ${TARGET_RELEASE_ELF}
${TARGET_RELEASE_ELF}: ${MAIN} ${TARGET_DIR}
	cargo objcopy --target=${CUSTOM_TRIPLE}.json --release -- -O binary ${TARGET_BIN}

# Create target directory for the bin
${TARGET_DIR}:
	mkdir -p ${TARGET_DIR}

# Delete target and binary files
clean:
	rm -rf ${TARGET_DIR}

# ==== UTILITY & DEBUG ======================================================= #
# Disassemble of the raw binary content (i386 ISA, Intel syntax)
bin: ${TARGET_BIN}
	objdump ${TARGET_BIN} -D -b binary -m i386 -M intel

# Disassemble content, read .comment ELF generated metadata section (useless)
elf: ${TARGET_RELEASE_ELF}
	objdump ${TARGET_RELEASE_ELF} -D -m i386 -M intel
	readelf -p .comment ${TARGET_RELEASE_ELF}