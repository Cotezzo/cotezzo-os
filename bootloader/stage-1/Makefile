# Define source assembly code files and binary outputs
MAIN=./src/bootloader.asm

# Define output binary location
TARGET_DIR=./target
TARGET_BIN=${TARGET_DIR}/stage-1.bin

# ==== TARGET ================================================================ #
# DEFAULT: build project if source has changed
all: ${TARGET_BIN}

# DEV & RELEASE: always clean and extract binary
dev: clean ${TARGET_BIN}
release: clean ${TARGET_BIN}

# Create stage-1 bootloader binary from assembly source
${TARGET_BIN}: ${MAIN} ${TARGET_DIR}
	nasm ${MAIN} -f bin -o ${TARGET_BIN}

# Create target directory for the bin
${TARGET_DIR}:
	mkdir -p ${TARGET_DIR}

# Delete target and binary files
clean:
	rm -rf ${TARGET_DIR}