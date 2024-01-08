; ==== MEMORY AND ARCH DIRECTIVES ================ ;
bits 16

; ==== SECTIONS AND EXPORTS ====================== ;
; Make the following methods visible to the linker
global _disk_reset
global _disk_read
global _disk_get_params

; Define the following code in the .text section, so
; that we can control its location via linker script
section .text

; ==== METHODS =================================== ;
;* Resets the disk controller (to use in case of
;* I/O errors).
;* Input parameters (from last pushed / left):
;* - Drive number
;* Output:
;* - Outcome of the operation (1 success, 0 error)
; This method implements the C calling convention.
_disk_reset:
    push bp                                         ; Save previous BP state
    mov bp, sp                                      ; Store method stack "start" - Pushing stuff would change SP
    ; No need to push DX, is caller saved

    mov dl, [bp+6]                                  ; 1^ Rust param: to reset drive

    stc                                             ; Reset CF to 1 to read the outcome of INT
    mov ah, 0x00                                    ; INT 13, 0: Reset Disk Controller
    int 0x13

    mov ax, 1                                       ; AX is the return value, it should reflect INT CF state
    sbb ax, 0                                       ; ax = ax - (0 + CF) (CF 0 -> AX 1, CF 1 -> AX 0)

    mov sp, bp                                      ; Restore SP
    pop bp                                          ; Restore BP

    retn                                            ; Return from near call


;* Reads from a given position on the disk and
;* loads the data to the given memory address.
;* Input parameters (from last pushed / left):
;* - Drive number (u8)
;* - Target Cylinder (u16)
;* - Target Head (u8)
;* - Target Sector (u8)
;* - Number of sectors to read (u8)
;* - Memory address where to load the data (* u8)
;* Output:
;* - Outcome of the operation (1 success, 0 error)
; This method implements the C calling convention.
_disk_read:

    ; Setup and save stack pointers
    push bp
    mov bp, sp
    push bx                                         ; BX is not caller saved
    push es                                         ; ES is not caller saved

    ; Parameters read and setup
    mov dl, [bp+8]                                  ; 1^ Rust param: to read drive, already set up for INT
    mov ax, [bp+12]                                 ; 2^ Rust param: target cylinder
    mov dh, [bp+16]                                 ; 3^ Rust param: target head, already set up for INT
    mov cl, [bp+20]                                 ; 4^ Rust param: target sector, already set up for INT

    mov ch, al                                      ; INT expects lower 8b of Cylinder value in CH
    shl ah, 6                                       ; INT expects upper 2b of Cylinder value in last CL's bits
    or cl, ah                                       ; OR to keep CL's lower 6 to previous value (target sector)
                                                    ; CX        = ---CH--- ---CL---
                                                    ; Cylinder  = 76543210 98
                                                    ; Sector    =            543210

    mov al, [bp+24]                                 ; 5^ Rust param: sectors to read, already set up for INT
    mov bx, [bp+28]                                 ; 6^ Rust param: loading address, already set up for INT
    mov es, bx                                      ; Far pointer, segment and offset are both pushed
    mov bx, [bp+32]


    ; Interrupt call
    stc                                             ; Reset CF to 1 to read the outcome of INT
    mov ah, 0x02
    int 0x13                                        ; INT 13, 2: Read Disk Sectors

    mov ax, 1                                       ; AX is the return value, it should reflect INT CF state
    sbb ax, 0                                       ; ax = ax - (0 + CF) (CF 0 -> AX 1, CF 1 -> AX 0)
    
    ; Restore registers and return
    pop es
    pop bx
    mov sp, bp
    pop bp
    retn


;* Uses BIOS to get informations about the disk and
;* store the retrieved informations in the given
;* memory addresses.
;* Input parameters (from last pushed / left):
;* - Drive number
;* - Output Drive Type Address (u8)
;* - Output Cylinder Address (u16)
;* - Output Head Address (u8)
;* - Output Sector Address (u8)
;* Output:
;* - Outcome of the operation (1 success, 0 error)
; This method implements the C calling convention.
_disk_get_params:
    push bp
    mov bp, sp
    push bx                                         ; BX is not caller saved
    push si                                         ; SI is not caller saved
    push di                                         ; DI is not caller saved
    push es                                         ; ES is not caller saved

    mov dl, [bp+6]                                  ; 1^ Rust param: to read drive

    stc                                             ; Reset CF to 1 to read the outcome of INT
    mov di, 0                                       ; INT 13, 8 requires ES:DI to be 0000:0000
    mov es, di
    mov ah, 0x08                                    ; INT 13, 8: Read Disk Parameters
    int 0x13

    pop es
    pop di

    ; INT 13,8 output:
    ; CF set on error
    ; AH = status (07h) (see #00234)
    ; CF clear if successful
    ; AH = 00h
    ; AL = 00h on at least some BIOSes
    ; BL = drive type (AT/PS2 floppies only)
    ; CH = low eight bits of maximum cylinder number
    ; CL = maximum sector number (5-0)
    ;      high two bits of cylinder number (7-6)
    ; DH = maximum head number
    ; DL = number of drives
    ; ES:DI -> drive parameter table (floppies only)

    jc .exit                                        ; If CF is set, error: jump to end without preparing output

    ; ==== TEST
    ;mov si, [bp+22]
    ;mov al, [si]
    ;add al, '0'
    ;mov bh, 0
    ;mov ah, 0x0E
    ;int 0x10
    ; ==== TEST

    mov si, [bp+10]                                 ; 2^ Rust param: drive type output address
    mov [si], bl
    mov bl, ch
    mov bh, cl
    shr bh, 6
    mov si, [bp+14]                                 ; 3^ Rust param: cylinders count output address
    mov [si], bx
    mov si, [bp+18]                                 ; 4^ Rust param: heads count output address
    mov [si], dh
    and cl, 0x3F
    mov si, [bp+22]                                 ; 5^ Rust param: sectors count output address
    mov [si], cl

    .exit:
    mov ax, 1                                       ; AX is the return value, it should reflect INT CF state
    sbb ax, 0                                       ; ax = ax - (0 + CF) (CF 0 -> AX 1, CF 1 -> AX 0)

    pop si
    pop bx
    mov sp, bp
    pop bp

    retn