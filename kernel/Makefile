# Define source assembly code files and binary outputs
MAIN=./src/kernel.asm

# Define output binary location
TARGET_DIR=./target
TARGET_BIN=${TARGET_DIR}/kernel.bin

# ==== TARGET ================================================================ #
# Don't treat these targets as files
.PHONY: all dev release clean

# DEFAULT: build project if source has changed
all: dev

# DEV & RELEASE: extract binary
dev: ${TARGET_BIN}
release: ${TARGET_BIN}

# Create stage-1 bootloader binary from assembly source
${TARGET_BIN}: ${MAIN} ${TARGET_DIR}
	nasm ${MAIN} -f bin -o ${TARGET_BIN}

# Create target directory for the bin
${TARGET_DIR}:
	mkdir -p ${TARGET_DIR}

# Delete target and binary files
clean:
	rm -rf ${TARGET_DIR}