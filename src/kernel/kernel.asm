; ==== CONSTANTS ============================================================= ;
%define LF 0x0D
%define CR 0x0A
%define START_TEXT `I'm the kernel!`

; ==== MEMORY AND ARCH DIRECTIVES ============================================ ;
; TODO: org 0x7C00
; TODO: bits 16

; ==== BOOTLOADER METHODS ==================================================== ;
start:              ; Jump to main label
    jmp main

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

; ==== BOOTLOADER MAIN ======================================================= ;
main:
    mov ax, 0
    mov ds, ax      ; Setup data segment register
    mov ss, ax      ; Setup stack segment register
    mov es, ax      ; Setup extra segment register
    mov sp, 0x7C00  ; Setup stack pointer register to start of the program

    mov si, start_text
    call print

    hlt             ; Stop executing

.halt:              ; Stop the execution to go any further
    jmp .halt

; ==== CONSTANT DATA DIRECTIVES ============================================== ;
start_text: db START_TEXT, LF, CR