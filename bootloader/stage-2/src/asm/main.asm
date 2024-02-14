
; ==== MEMORY AND ARCH DIRECTIVES ============================================================================ ;
bits 16

; ==== GLOBALS AND EXTERN METHODS ============================================================================ ;
; Make the following methods visible to the linker.
global _start

; Declare Rust entry method to call after switching from 16rm to 32pm.
extern _rs_start

; ==== ENTRY POINT =========================================================================================== ;
; Define entry point code in the .text section, so that we can control its location via linker script
; Required since we need to tell the entry point name to the linker.
; "_start" is the default entry point name for most systems.
section .text.start

_start:
    [bits 16]

    mov [drive_number], dl                          ; Save disk number value from stage-1

    ; Switching from 16rm to 32pm
    cli                                             ; Disable interrupts
    call a20_check_and_enable                       ; Check and enable a20
    lgdt [gdt_descriptor]                           ; Load GDT into GDTR register

    mov eax, cr0                                    ; Read control register 0 value
    or al, 1                                        ; Set first bit (protected mode)
    mov cr0, eax                                    ; Update CR0 value

    ; From here onwards, segments refer to GDT
    ; Setup Data Segment selector
    mov ax, gdt.selector_32pm_ds - gdt
    mov ds, ax
    mov ss, ax
    mov es, ax

    ; Setup Code Segment selector
    ; Stage-2 (this code) is loaded with segments set to 0x0 and with offset 0x500. This way, the segmented
    ; offset is equal to the physical offset, which itself is equal to the address in the code segment
    ; selector, since the latter defines a flat memory model (base is 0).
    ; This way, there is no need to manipulate the address:
    ; for example, if using segment 0x1000 and offset 0x0, the physical address would be 0x10000.
    ; You would need to jump at the "real" address of .32pm label (0x10030 instead of 0x1000:0x30, for example).
    ; This manipulation is quite challenging and I couldn't solve it, so I opted for 0x0:0x500 instead.
    jmp dword gdt.selector_32pm_cs - gdt:.32pm

    .32pm:
    [bits 32]
    sti                                             ; Re-enable interrupts

    ;!"Wrapping around" (in 32bit) doesn't always work: 0xFFFFFFFF isn't always a valid memory address.
    ; TODO: dynamically set to ESP the maximum RAM address before calling _rs_start.
    ;! Setting ESP to a value > 0xFFFF is not possible here, since we occasionally return to real mode to
    ;! call BIOS interrupts for disk I/O operations. Keep value set by stage-1, ~0xFFFF.
    ;mov esp, 0x800_0000                            ; RAM size here is 128MB (134_217_728 or 0x800_0000 byte)
    
    push dword [drive_number]                       ; Push disk number as first _rs_start parameter
    call _rs_start                                  ; Call _rs_start method declared in Rust and linked
    ;//push dword 0xDEADBEEF                        ; Push fake EPC register so that our real parameter is read
    ;//jmp dword gdt.selector_32pm_cs-gdt:_rs_start ; Jump to 32-bit Rust block


; ==== METHODS =============================================================================================== ;
; ==== PRINT ===================================== ;
; Define special characters and strings to print.
%define LF 0x0D
%define CR 0x0A

;* Prints a string stored in memory to TTY.
;* Input:
;* - SI: string memory address
print16r:
    [bits 16]
    push ax                                         ; Store AX
    push bx                                         ; Store BX
    push si                                         ; Store SI

    mov ah, 0x0E                                    ; Interrupt method: 10, E: Write Character in TTY
    mov bh, 0                                       ; Set page number used by int
    .loop:                                          ; Loop start
        lodsb                                       ; Load content of ds:[si] in AL, inc SI
        test al, al                                 ; Check whether the character was NULL (last one)
        jz .end                                     ; If it is (Z flag is set since AL is 0), exit

        int 0x10                                    ; Call int and display the character in AL
        jmp .loop

    .end:
    pop si                                          ; Restore previous SI value
    pop bx                                          ; Restore previous BX value
    pop ax                                          ; Restore previous AX value
    ret                                             ; If Z flag is set, return (pop ip)

; ==== KEYBOARD CONTROLLER (PS/2 CONTROLLER) ===== ;
; Contorller ports and commands
%define PS2_PORT_DATA 0x60
%define PS2_PORT_COMMAND 0x64
%define PS2_COMMAND_DISABLE_KEYBOARD 0xAD
%define PS2_COMMAND_ENABLE_KEYBOARD 0xAE
%define PS2_COMMAND_READ_FROM_OUTPUT 0xD0
%define PS2_COMMAND_WRITE_TO_OUTPUT 0xD1

;* Checks if the A20 Line is enabled. If it's not,
;* tries to enable it.
a20_check_and_enable:
    [bits 16]

    call a20_check                                  ; Check A20 Line state
    cmp ax, 1

    ; If A20 is disabled, enable it and check again
    je .a20_enabled
    call a20_enable
    call a20_check
    cmp ax, 1
    je .a20_enabled

    ; If the A20 is still disabled, halt execution
    mov si, text16r_a20_disabled
    call print16r
    hlt

    ; If the A20 is enabled, go on with switching
    .a20_enabled:
    ret

;* Checks if the A20 Line is enabled.
;* Old BIOSes might not enable it by default.
;* Output:
;* - AX, 1 if the A20 line is enabled, 0 otherwise.
a20_check:
    [bits 16]
    push es
    push di
    push ds
    push si
    
    ; Setup segments and index regs for mem access
    xor ax, ax
    mov es, ax
    mov di, 0x7DFE

    not ax
    mov ds, ax
    mov si, 0x7E0E

    push word [es:di]                               ; Save previous values
    push word [ds:si]
    mov byte [es:di], 0x00                          ; Clear previous 0:0x7DFE value with v1
    mov byte [ds:si], 0xFF                          ; Write v2 (different from v1) to FFFF:0x7E0E (0:7DFE + 1MB)
    cmp byte [es:di], 0xFF                          ; Compare 0:0x7DFE against v2
    pop word [ds:si]
    pop word [es:di]                                ; Restore previous values

    mov ax, 1
    jne .exit                                       ; Byte NOT equal, memory does NOT wrap (A20 enabled), ret 1
    xor ax, ax                                      ; Byte matches, memory wraps (A20 disabled), ret 0
    .exit:

    pop si
    pop ds
    pop di
    pop es
    ret

;* Uses the hardware I/O ports on the Keyboard 
;* Controller to try enabling the A20 Gate.
a20_enable:
    [bits 16]
    push ax

    ;! Before writing to ibuf, wait for it to be empty.

    ; Disable keyboard to avoid interruptions.
    call ps2_wait_ibuf_empty                        ; Wait for the input buffer to be empty
    mov al, PS2_COMMAND_DISABLE_KEYBOARD
    out PS2_PORT_COMMAND, al

    ; Read configuration byte
    call ps2_wait_ibuf_empty                        ; Wait for the input buffer to be empty
    mov al, PS2_COMMAND_READ_FROM_OUTPUT
    out PS2_PORT_COMMAND, al
    call ps2_wait_obuf_full                         ; Wait for the output buffer to be full
    in al, PS2_PORT_DATA
    push ax                                         ; Save AL for later, it is used for I/O operations

    ; Write modified configuration byte
    call ps2_wait_ibuf_empty                        ; Wait for the input buffer to be empty
    mov al, PS2_COMMAND_WRITE_TO_OUTPUT
    out PS2_PORT_COMMAND, al
    call ps2_wait_ibuf_empty                        ; Wait for the input buffer to be empty
    pop ax                                          ;! pop AFTER waiting, since wait does not restore AL value
    or al, 2
    out PS2_PORT_DATA, al

    ; Re-enable keyboard
    call ps2_wait_ibuf_empty                        ; Wait for the input buffer to be empty
    mov al, PS2_COMMAND_ENABLE_KEYBOARD
    out PS2_PORT_COMMAND, al

    call ps2_wait_ibuf_empty                        ; Wait for the process to end
    pop ax
    ret

;* Waits until the PS/2 input buffer can be used.
;! Uses AX register without restoring its state.
ps2_wait_ibuf_empty:
    [bits 16]
    in al, PS2_PORT_COMMAND                         ; Get controller state from status register
    test al, 2                                      ; If the second bit is not set, the buffer is empty, exit
    jnz ps2_wait_ibuf_empty                         ; If the buffer is full, keep waiting
    ret

;* Waits until the PS/2 output buffer is full.
;! Uses AX register without restoring its state.
ps2_wait_obuf_full:
    [bits 16]
    in al, PS2_PORT_COMMAND                         ; Get controller state from status register
    test al, 1                                      ; If the first bit is set, the buffer is full, exit
    jz ps2_wait_obuf_full                           ; If the buffer is empty, keep waiting
    ret

; ==== DATA ================================================================================================== ;
section .data
drive_number:                                       ; Store drive number value to pass to Rust module
    dd 0

section .rodata
; Print method strings
text16r_test: db `Real!`, LF, CR, 0
text16r_a20_enabling: db `Enabling A20 Line...`, LF, CR, 0
text16r_a20_disabled: db `Could not enable A20 Line`, LF, CR, 0
text16r_a20_enabled: db `A20 Line Enabled`, LF, CR, 0

; ==== GDT =================================================================================================== ;
section .rodata.gdt
; Global Descriptor Table
gdt:
    .selector_empty:                                ;* 1st: must be empty
        dq 0

    .selector_32pm_cs:                              ;* 2nd: 32b code, flat memory model
        dw 0xFFFF                                   ; Limit (0-15 bits)
        dw 0                                        ; Base (0-15 bits)
        db 0                                        ; Base (16-23 bits)
        db 1_00_1_1_0_1_0b                          ; Present, Ring, Type, Exec, Direct/Confirm, R/W, Accessed
        db 1_1_0_0_1111b                            ; Granularity, Size, Long-Mode, Reserved, Limit (16-19 bits)
        db 0                                        ; Base (24-31 bits)

    .selector_32pm_ds:                              ;* 3rd: 32b data, flat memory model
        dw 0xFFFF
        dw 0
        db 0
        db 1_00_1_0_0_1_0b                          ; Like 32bit code segment, but Executable bit set to 0
        db 1_1_0_0_1111b
        db 0

    .selector_16pm_cs:                              ;* 4th: 16b code, flat memory model, used for 32pm_to_16rm
        dw 0xFFFF
        dw 0
        db 0
        db 1_00_1_1_0_1_0b
        db 0_0_0_0_1111b                            ; Like 32bit code segment, but Granularity and Size set to 0
        db 0

    .selector_16pm_ds:                              ;* 5th: 16b data, flat memory model, used for 32pm_to_16rm
        dw 0xFFFF
        dw 0
        db 0
        db 1_00_0_0_0_1_0b                          ; Like 16bit code segment, but Executable bit set to 0
        db 0_0_0_0_1111b
        db 0
        
gdt_descriptor:
    dw gdt_descriptor - gdt - 1                     ; GDT size - 1 (16b)
    dd gdt                                          ; GDT address (32b)