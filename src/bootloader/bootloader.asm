; ==== CONSTANTS (MACRO) ===================================================== ;
%define LF 0x0D
%define CR 0x0A
%define DISK_ERROR_TEXT `Il tuo floppy fa un po' cagare`
%define REBOOTING_TEXT `Rebooting...`
%define DATA_LOAD_TEXT `Data successfully loaded`

%define BOOT_MEM_START 0x7C00
%define DATA_MEM_START 0x7C00 + 0x0200

; ==== MEMORY AND ARCH DIRECTIVES ============================================ ;
org BOOT_MEM_START  ; Bootloader loaded in 0x7C00 (32kB - 512b - 512b)
bits 16             ; Bootloader runs in 16b mode for retro and space

; ==== FAT12 HEADERS - JUMP ================================================== ;
jmp short boot      ; Jump to bootloader so that the BIOS doesn't read headers
nop                 ; Do nothing for 1 clock tick

; ==== FAT12 HEADERS - BPB (BIOS Parameter Block) ============================ ;
db 'MSWIN4.1'       ; OEM name/ver, useless (MS recommends MSWIN4.1)
dw 512              ; bytes per sector: 512B (floppy)
db 1                ; Sectors per cluster
dw 1                ; Reserved sectors (1: this one)
db 2                ; FATs (File Allocation Table) number (2 for rendundancy)
dw 224              ; Root entries (FAT12: 224, FAT16: 512 recomm, FAT32: 0)
dw 2880             ; Sector number (we defined 2880 sectors, 512B each)
db 0xF0             ; Media descriptor (1.44MB Floppy: F0, Hard Disk: F8...)
dw 9                ; Number of FAT (Table) sectors (FAT12/16: 9, FAT32: 0)
bpb_sectors: dw 18  ; Sectors per track
bpb_heads: dw 2     ; Diskette heads
dd 0                ; Number of hidden sectors (preceding the partition)
dd 0                ; Set if sector number is > 65535 (sector number set to 0)

; ==== FAT12 HEADERS - EBR (Extended Boot Record) ============================ ;
ebp_drive: db 0     ; Drive number, useless (Floppy: 0, Hard Disk: 8...)
db 0                ; Reserved
db 0x29             ; Signature (0x28 or 0x29)
dd 0x10101010       ; Serial number, useless
db 'LETS FKN GO'    ; Volume label, useless
db 'FAT12   '       ; System identifier

; ==== BOOTLOADER METHODS ==================================================== ;
boot:               ; Set there so that the FAT12 short jump is in range
    jmp main        ; Jump to actual bootloader main

; Print a string stored in memory to STDOUT
; Input:
; . SI: string memory address
print:
    push ax         ; Store AX
    push bx         ; Store BX
    push si         ; Store SI

    mov ah, 0x0E    ; Interrupt method: 10, E: Write Character in TTY
    mov bh, 0       ; Set page number used by int
    .loop:          ; Loop start
        lodsb       ; Load content of ds:[si] in AL, inc SI
        int 0x10    ; Call int and display the character in AL
        xor al, CR  ; Check whether the character was CR (last one)
                    ; If not, the Z flag is set to 0 (result != 0)
        jnz .loop   ; If Z flag is not set (j not z), keep looping

    pop si          ; Restore previous SI value
    pop bx          ; Restore previous BX value
    pop ax          ; Restore previous AX value
    ret             ; If Z flag is set, return (pop ip)

; Converts LBA to CHS address, restores AX, DX to state previous to method call.
; Input:
; . AX: LBA address to be converted
; Outputs the CHS address directly where the BIOS expects it (CX, DX (DH)):
; . Sector:   CX [0-5 bits]       CX        = ---CH--- ---CL---
; . Cylinder: CX [6-15 bits]      Cylinder  = 76543210 98
;   2 byte più a sx a fine CL     Sector    =            543210
; . HEAD:     DH [0-5 bits]
lba_to_chs:
    push ax                 ; Store AX value to stack
    push dx                 ; Store DX value to stack
                            ; (Only DL is needed, but we can't push only 8b)

    xor dx, dx              ; Clear DX value (used for word division)
    div word [bpb_sectors]  ; AX = LBA / Sectors
                            ; DX = LBA % Sectors
    inc dx                  ; LBA % Sectors + 1 --> S
    mov cx, dx              ; Save S to lower 8b of CX (upper 2 will be lost)
    xor dx, dx              ; Clear DX value (AX contains the previous result)
    div word [bpb_heads]    ; AX = (LBA / Sectors) / Heads --> C
                            ; DX = (LBA / Sectors) % Heads --> H
    mov dh, dl              ; Save H (DL contains the lower 8 bits of DX)
    mov ch, al              ; Save C (Lower 8b of AX to the upper 8b of CX)
    shl ah, 6               ; Save C (Upper 2b of AH, which we shifted)
    or cl, ah               ; OR to keep CL's lower 6 to previous value (S))

    pop ax                  ; Retrieve DX value from stack (in AX temporarily)
    mov dl, al              ; Only restore DL (DX lower 8b), DH has our output
    pop ax                  ; Retrieve and restore AX value from stack
    ret

; Resets disk controller, used before trying a read operation
; Input:
; . DL: Drive number
disk_reset:
    pusha               ; Save all registers
    stc                 ; Reset CF to 1, some BIOSes don't do that
    mov ah, 0x00        ; Interrupt method: 13, 0: Reset Disk Controller
    int 0x13            ; Reset controller
    jc disk_error       ; If the CF is set, there was a disk error
    popa                ; Restore all registers
    ret

; Retrieves data from the hard drive and loads it to memory.
; Input:
; . AX: LBA (Physical sector location)
; . CL: Sectors to read (max 128)
; . DL: Drive number
; . ES:BX: RAM address where to store read data
disk_read:
    push ax             ; Store registers modified to restore them later
    push cx
    push dx             ; Used for lba_to_chs output
    push di

    push cx             ; Store CL to stack (lba_to_chs would overwrite it)
    call lba_to_chs     ; Convert LBA address
                        ; lba_to_chs already setups CH, CL, DH for the INT call
                        ; We don't need AX anymore, it's safe to use AH, AL
    pop ax              ; Retrieve CL from stack to AL (INT uses AL)
    mov ah, 0x02        ; Select interrupt method: 13, 2: Read Disk Sectors

    mov di, 3           ; Store in DI the retry count
    .retry:             ; Floppy are unreliable, recommended to try read 3 times
        pusha           ; Push ALL registers, don't know what BIOS will do
        stc             ; Reset CF (Carry Flag) to 1, some BIOSes don't do that
        int 0x13        ; BIOS Interrupt to read from disk
                        ; We use the flag to determine the outcome of the read
        popa            ; Restore all registers
        jnc .done       ; If the flag is cleared, read success, exit the loop

        call disk_reset ; If read failed, reset disk controller before retrying
        dec di          ; Decrement retry counter
        test di, di     ; Bitwise AND, discards result and sets flags
        jnz .retry      ; If the 0 (Z) flag is not set, keep trying

    .fail:              ;? Is this label useful?
        jmp disk_error  ; Attempts exhausted, display error and restarting

    .done:
        pop di          ; Restore modified registers
        pop dx
        pop cx
        pop ax
        ret             ; Exit method


; ==== BOOTLOADER MAIN ======================================================= ;
main:
    mov ax, 0
    mov ds, ax              ; Setup data segment register
    mov ss, ax              ; Setup stack segment register
    mov es, ax              ; Setup extra segment register
    mov sp, BOOT_MEM_START  ; Setup stack pointer register to program's start

    ; AX, CL, DL, ES:BX
    ;mov dl, [ebp_drive]    ; Select the drive to read from, BIOS already set it
    mov ax, 1               ; Read data in the second sector of the disk
    mov cl, 1               ; Only read 1 sector
    mov bx, DATA_MEM_START  ; Place data after the bootloader (0x0200 = 512b)
    call disk_read          ; Load data to memory

    mov si, data_load_text  ; Load success message address
    call print              ; Display message

    cli                     ; Disable interrupts: CPU can't exit of halt state
    hlt                     ; Stop executing

disk_error:                 ; Don't care about pushing anything, we're rebooting
    mov si, disk_error_text ; Load disk error message address
    call print              ; Display error message

    mov ah, 0               ; Select interrupt method: 16, 0: Wait for keypress
    int 0x16                ; Wait for the user to press a key

    mov si, rebooting_text  ; Load rebooting message address
    call print              ; Display rebooting message

    jmp 0xFFFF:0            ; Jump to BIOS to reboot (mapped to read from ROM) [https://superuser.com/questions/988473/why-is-the-first-bios-instruction-located-at-0xfffffff0-top-of-ram]
    
    ;cli                     ; Disable interrupts: CPU can't exit of halt state
    ;hlt                     ; Stop executing

; ==== CONSTANT DATA DIRECTIVES ============================================== ;
disk_error_text: db DISK_ERROR_TEXT, LF, CR
rebooting_text: db REBOOTING_TEXT, LF, CR
data_load_text: db DATA_LOAD_TEXT, LF, CR

; ==== PADDING AND SIGNATURE ================================================= ;
times 510-($-$$) db 0       ; Padding until sector end - 2 bytes for signature
dw 0xAA55                   ; Signature at the end of the sector (magic number)