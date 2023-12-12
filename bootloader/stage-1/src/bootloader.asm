; ==== CONSTANTS (MACRO) ===================================================================================== ;
%define LF 0x0D
%define CR 0x0A
%define DISK_ERROR_TEXT `Il tuo floppy fa un po' cagare`
%define REBOOTING_TEXT `Reboot...`
%define STAGE_2_BIN `STAGE-2 BIN`
;%define STAGE_2_BIN `KERNEL  BIN`

%define BOOT_MEM_OFFSET 0x7C00
%define DATA_MEM_OFFSET 0x7C00 + 0x0200
%define KERNEL_MEM_SEGMENT 0x1000
%define KERNEL_MEM_OFFSET 0

; ==== MEMORY AND ARCH DIRECTIVES ============================================================================ ;
org BOOT_MEM_OFFSET                                 ; Bootloader loaded in 0x7C00 (32kB - 512b - 512b)
bits 16                                             ; Bootloader runs in 16b mode for retro and space

; ==== FAT12 HEADERS - JUMP ================================================================================== ;
jmp short boot                                      ; Jump to bootloader so that the BIOS doesn't read headers
nop                                                 ; Do nothing for 1 clock tick

; ==== FAT12 HEADERS - BPB (BIOS Parameter Block) ============================================================ ;
db 'MSWIN4.1'                                       ; OEM name/ver, useless (MS recommends MSWIN4.1)
bpb_bytes_per_sector: dw 512                        ; bytes per sector: 512B (floppy)
bpb_sectors_per_cluster: db 1                       ; Sectors per cluster
bpb_reserved_sectors: dw 1                          ; Reserved sectors (1: this one)
bpb_fat_count: db 2                                 ; FATs (File Allocation Table) number (2 for rendundancy)
bpb_root_entries: dw 224                            ; Root entries (FAT12: 224, FAT16: 512 recomm, FAT32: 0)
dw 2880                                             ; Sector number (we defined 2880 sectors, 512B each)
db 0xF0                                             ; Media descriptor (1.44MB Floppy: F0, Hard Disk: F8...)
bpb_sectors_per_fat: dw 9                           ; Number of FAT (Table) sectors (FAT12/16: 9, FAT32: 0)
bpb_sectors: dw 18                                  ; Sectors per track
bpb_heads: dw 2                                     ; Diskette heads
dd 0                                                ; Number of hidden sectors (preceding the partition)
dd 0                                                ; Set if sector number is > 65535 (sector number set to 0)

; ==== FAT12 HEADERS - EBR (Extended Boot Record) ============================================================ ;
ebp_drive: db 0                                     ; Drive number, useless (Floppy: 0, Hard Disk: 8...)
db 0                                                ; Reserved
db 0x29                                             ; Signature (0x28 or 0x29)
dd 0x10101010                                       ; Serial number, useless
db 'LETS FKN GO'                                    ; Volume label, useless
db 'FAT12   '                                       ; System identifier

; ==== BOOTLOADER METHODS ==================================================================================== ;
boot:                                               ; Set there so that the FAT12 short jump is in range
    jmp main                                        ; Jump to actual bootloader main

; Print a string stored in memory to STDOUT
; Input:
; - SI: string memory address
print:
    push ax                                         ; Store AX
    push bx                                         ; Store BX
    push si                                         ; Store SI

    mov ah, 0x0E                                    ; Interrupt method: 10, E: Write Character in TTY
    mov bh, 0                                       ; Set page number used by int
    .loop:                                          ; Loop start
        lodsb                                       ; Load content of ds:[si] in AL, inc SI
        int 0x10                                    ; Call int and display the character in AL
        xor al, CR                                  ; Check whether the character was CR (last one)
                                                    ; If not, the Z flag is set to 0 (result != 0)
        jnz .loop                                   ; If Z flag is not set (j not z), keep looping

    pop si                                          ; Restore previous SI value
    pop bx                                          ; Restore previous BX value
    pop ax                                          ; Restore previous AX value
    ret                                             ; If Z flag is set, return (pop ip)

; Converts LBA to CHS address, restores AX, DX to state previous to method call.
; Input:
; - AX: LBA address to be converted
; Outputs the CHS address directly where the BIOS expects it (CX, DX (DH)):
; - Sector:   CX [0-5 bits]       CX        = ---CH--- ---CL---
; - Cylinder: CX [6-15 bits]      Cylinder  = 76543210 98
;   2 byte più a sx a fine CL     Sector    =            543210
; - HEAD:     DH [0-5 bits]
lba_to_chs:
    push ax                                         ; Store AX value to stack
    push dx                                         ; Store DX value to stack
                                                    ; (Only DL is needed, but we can't only push 8b)

    xor dx, dx                                      ; Clear DX value (used for word division)
    div word [bpb_sectors]                          ; AX = LBA / Sectors
                                                    ; DX = LBA % Sectors
    inc dx                                          ; LBA % Sectors + 1 --> S
    mov cx, dx                                      ; Save S to lower 8b of CX (upper 2 will be lost)
    xor dx, dx                                      ; Clear DX value (AX contains the previous result)
    div word [bpb_heads]                            ; AX = (LBA / Sectors) / Heads --> C
                                                    ; DX = (LBA / Sectors) % Heads --> H
    mov dh, dl                                      ; Save H (DL contains the lower 8 bits of DX)
    mov ch, al                                      ; Save C (Lower 8b of AX to the upper 8b of CX)
    shl ah, 6                                       ; Save C (Upper 2b of AH, which we shifted)
    or cl, ah                                       ; OR to keep CL's lower 6 to previous value (S))

    pop ax                                          ; Retrieve DX value from stack (in AX temporarily)
    mov dl, al                                      ; Only restore DL (DX lower 8b), DH has our output
    pop ax                                          ; Retrieve and restore AX value from stack
    ret

; Resets disk controller, used before trying a read operation
; Input:
; - DL: Drive number
disk_reset:
    pusha                                           ; Save all registers
    stc                                             ; Reset CF to 1, some BIOSes don't do that
    mov ah, 0x00                                    ; Interrupt method: 13, 0: Reset Disk Controller
    int 0x13                                        ; Reset controller
    jc disk_error                                   ; If the CF is set, there was a disk error
    popa                                            ; Restore all registers
    ret

; Retrieves data from the hard drive and loads it to memory.
; Input:
; - AX: LBA (Physical sector location)
; - CL: Sectors to read (max 128)
; - DL: Drive number
; - ES:BX: RAM address where to store read data
disk_read:
    push ax                                         ; Store registers modified to restore them later
    push cx
    push dx                                         ; Used for lba_to_chs output
    push di

    push cx                                         ; Store CL to stack (lba_to_chs would overwrite it)
    call lba_to_chs                                 ; Convert LBA address
                                                    ; lba_to_chs already setups CH, CL, DH for the INT call
                                                    ; We don't need AX anymore, it's safe to use AH, AL
    pop ax                                          ; Retrieve CL from stack to AL (INT uses AL)
    mov ah, 0x02                                    ; Select interrupt method: 13, 2: Read Disk Sectors

    mov di, 3                                       ; Store in DI the retry count
    .retry:                                         ; Floppy are unreliable, recommended to try read 3 times
        pusha                                       ; Push ALL registers, don't know what BIOS will do
        stc                                         ; Reset CF (Carry Flag) to 1, some BIOSes don't do that
        int 0x13                                    ; BIOS Interrupt to read from disk
                                                    ; We use the flag to determine the outcome of the read
        popa                                        ; Restore all registers
        jnc .done                                   ; If the flag is cleared, read success, exit the loop

        call disk_reset                             ; If read failed, reset disk controller before retrying
        dec di                                      ; Decrement retry counter
        test di, di                                 ; Bitwise AND, discards result and sets flags
        jnz .retry                                  ; If the 0 (Z) flag is not set, keep trying

    .fail:              ;? Is this label useful?
        jmp disk_error                              ; Attempts exhausted, display error and restarting

    .done:
        pop di                                      ; Restore modified registers
        pop dx
        pop cx
        pop ax
        ret                                         ; Exit method

; Wait for the user to press any key, then returns and continues execution
wait_keypress:
    push ax                                         ; Store AX
                                                    ; INT 16, 0 returns values in AX
    mov ah, 0                                       ; Select interrupt method: 16, 0: Wait for keypress
    int 0x16                                        ; Wait for the user to press a key
    pop ax                                          ; Restore AX
    ret

; ==== BOOTLOADER MAIN ======================================================================================= ;
main:
    ; ==== SEGMENTS SETUP ======================== ;
    mov ax, 0
    mov ds, ax                                      ; Setup data segment register
    mov ss, ax                                      ; Setup stack segment register
    mov es, ax                                      ; Setup extra segment register
    ; BIOS could initialize CS:IP to 0x07C0:0, but can't directly set CS value:
    ; Perform a jump to override the CS:IP values
    jmp dword 0x0000:._
    ._:
    mov sp, BOOT_MEM_OFFSET                         ; Setup stack pointer register to program's start

    ; ==== LOAD BIOS DISK INFO =================== ;
    ; To be sure, read disk informations directly from BIOS interrupt
    push es                                         ; Save ES value

    mov ah, 8                                       ; INT 13, 8 returns disk informations
    int 0x13                                        ; DL is already set by BIOS with the correct drive
    jc disk_error                                   ; If the CF is set, there was a disk error
    and cx, 0x003F                                  ; Sector count is contained in lower 6 bits
    mov [bpb_sectors], cx                           ; Override previous value
    inc dh                                          ; Heads count starts from 0, add 1 to get real value
    mov [bpb_heads], dh                             ; Override previous value

    pop es                                          ; Restore previous ES value (13,8 changes it)

    ; ==== LOAD ROOT DIRECTORY =================== ;
    ; Calculate Root Directory Size
    ; Assuming a entry has a 32B size, you can directly calculate sector size with bpb_root_entries / 16
    ; Entries: 224, Size 32B, Total Sector Size: 14
    ; 224*32 / 512 = 224 / 16 = 14
    xor dx, dx                                      ; Clear DX value (used for word division)
    mov ax, [bpb_root_entries]
    mov cx, 16
    div cx                                          ; AX = AX / 16 = sectors (without accounting for remainder)
    test dx, dx                                     ; Check if the remainder is > 0
    jz .continue                                    ; If not, skip next line
    inc ax                                          ; If so, add 1 to the number of sectors
    .continue:
    mov cx, ax                                      ; Number of sectors to read from disk

    ; Calculate Root Directory Entries LBA
    mov ax, [bpb_sectors_per_fat]
    xor bx, bx
    mov bl, [bpb_fat_count]
    mul bx                                          ; ax = ax * operand - ax = fats total sector size
    add ax, [bpb_reserved_sectors]                  ; root start = reserved sects + fat sects

    ; ES segment already set
    mov dl, [ebp_drive]                             ; Select the drive to read from, BIOS already set it
    mov bx, DATA_MEM_OFFSET                         ; Place data after the bootloader (0x0200 = 512b)
    call disk_read                                  ; Load Root Entries to memory
    ;! Instead of loading the whole root directory, one could load one sector at a time
    ;! Not really an issue for now since we have from 0x00007E00 to 0x0007FFFF "free" in real mode
    ;! (https://wiki.osdev.org/Memory_Map_(x86))

    ; Save cluster region offset for later (root offset + root size)
    add ax, cx
    mov [cluster_region_offset], ax

    ; ==== READ ROOT ENTRIES ===================== ;
    xor ax, ax                                      ; Checked entries counter - starts at 0
    .read_root_entries:
        mov cx, [bx]                                ; Entry starts with file name, load first byte
        test cx, cx                                 ; If first B is 0, we reached the last one: print error
        jz disk_error                               ; If not, check if it's our kernel
        
        mov si, stage_2_file_name                    ; Load kernel filename string address
        mov di, bx                                  ; Load current entry filename address
        mov cx, 11                                  ; Filename size (11 chars for FAT12)
        repe cmpsb                                  ; Check if filename matches kernel's
        jz .kernel_found                            ; If it matches, load clusters, if not, go on

        inc ax                                      ; Increment counter
        add bx, 32                                  ; Increment address to next entry (entry = 32B)
        cmp ax, [bpb_root_entries]                  ; Check if we reached the last root entry
        jnz .read_root_entries                      ; If not, keep searching
        jmp disk_error                              ; If so, end of root reached, print error

    ; ==== LOAD FAT ============================== ;
    ; BX points to the correct root entry - lower cluster value is at 27th, 28th byte
    .kernel_found:
        mov bx, [bx + 26]
        mov [current_cluster], bx                   ; Load word at 27th byte - lower cluster
        
        mov ax, [bpb_reserved_sectors]              ; Read FAT, right after reserved sector
        mov cx, [bpb_sectors_per_fat]               ; Read whole FAT
        ; ES, DL already set
        mov bx, DATA_MEM_OFFSET                     ; Place data after the bootloader (override root entries)
        call disk_read                              ; Load first FAT to memory
    
    ; ==== LOAD KERNEL BINARY ==================== ;
    mov bx, KERNEL_MEM_SEGMENT                      ; Place Kernel at the designated memory area
    mov es, bx                                      ; Load to 0x10000 in order to leave room for
    mov bx, KERNEL_MEM_OFFSET                       ; the FAT table currently saved in RAM
    .load_kernel_cluster:
        ; Calculate current cluster sector
        mov ax, [current_cluster];7d41              ; Prepare first mul operand
        sub ax, 2                                   ; Subtract 2 to cluster, skipping the first two FAT entries
        xor cx, cx                                  ; Clean CH since the whole CX is used (bug fixed using gdb)
        mov cl, [bpb_sectors_per_cluster]           ; Prepare second mul operand
        mul cx
        add ax, [cluster_region_offset]             ; Retrieve cluster region start saved earlier

        mov cl, [bpb_sectors_per_cluster]           ; Cluster size
        mov dl, [ebp_drive];7d55                    ; Select the drive to read from, BIOS already set it
        call disk_read                              ; Load cluster to memory

        ; Read next cluster from FAT
        ; FAT entry is at DATA_MEM_OFFSET + current_cluster * 3 / 2
        xor dx, dx                                  ; Clear DX for division
        mov ax, [current_cluster]                   ; Prepare first mul operand
        mov cx, 3                                   ; Prepare second mul operand (can't use constants)
        mul cx                                      ; AX = AX * CX
        mov cx, 2                                   ; Prepare second div operand
        div cx                                      ; AX = AX / CX - AX: result, DX: reminder
        mov si, ax;7d66                             ; Can't use AX to access RAM, move FAT entry index to SI
        mov ax, [DATA_MEM_OFFSET + si]              ; Store next cluster in AX - FAT RAM location + index

        ; Adjust read bytes to match 12b data
        test dx, dx
        jz .even
        shr ax, 4                                   ; If the remainder of / 2 is 1, take upper 12 bytes
        .even:                                      ; If the remainder is 0, take lower 12 bytes
        and ax, 0x0FFF                              ; No need to "avoid" doing it if it's 1

        cmp ax, 0x0FF8                              ; If the next cluster is >= FF8, this was the last cluster:
        jae .start_kernel                           ; jump to kernel startup

        mov [current_cluster], ax                   ; Setup current cluster for next iteration
        mov ax, [bpb_bytes_per_sector]              ; Prepare first mul operand
        mul word [bpb_sectors_per_cluster]          ; Calculate cluster size
        add bx, ax                                  ; Save next cluster after the previous to avoid overrides
        jmp .load_kernel_cluster                    ; Load new current cluster

    .start_kernel:
        mov ax, KERNEL_MEM_SEGMENT
        mov ds, ax                                  ; Setup data segment register for the kernel
        mov ss, ax                                  ; Setup stack segment register for the kernel
        mov es, ax                                  ; Setup extra segment register for the kernel
        mov sp, KERNEL_MEM_OFFSET                   ; Setup stack pointer register to kernel's start
        jmp KERNEL_MEM_SEGMENT:KERNEL_MEM_OFFSET    ; Far jump to loaded kernel code

    cli                                             ; Disable interrupts: CPU can't exit of halt state
    hlt                                             ; Stop executing

disk_error:                                         ; Don't care about pushing anything, we're rebooting
    mov si, disk_error_text                         ; Load disk error message address
    call print                                      ; Display error message
    call wait_keypress                              ; Wait for a keypress before rebooting
    mov si, rebooting_text                          ; Load rebooting message address
    call print                                      ; Display rebooting message

    jmp 0xFFFF:0                                    ; Jump to BIOS to reboot (mapped to read from ROM) [https://superuser.com/questions/988473/why-is-the-first-bios-instruction-located-at-0xfffffff0-top-of-ram]
    
    ;cli                                            ; Disable interrupts: CPU can't exit of halt state
    ;hlt                                            ; Stop executing

; ==== CONSTANT DATA DIRECTIVES ============================================================================== ;
disk_error_text: db DISK_ERROR_TEXT, LF, CR
rebooting_text: db REBOOTING_TEXT, LF, CR
stage_2_file_name: db STAGE_2_BIN

; ==== VARIABLE DATA DIRECTIVES ============================================================================== ;
cluster_region_offset: dw 0
current_cluster: dw 0

; ==== PADDING AND SIGNATURE ================================================================================= ;
times 510-($-$$) db 0                               ; Padding until sector end - 2 bytes for signature
dw 0xAA55                                           ; Signature at the end of the sector (magic number)