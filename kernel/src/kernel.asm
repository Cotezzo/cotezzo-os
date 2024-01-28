; ==== CONSTANTS ================================================================================= ;
%define LF 0x0D
%define CR 0x0A
%define START_TEXT `I'm the big kernel!`

; ==== MEMORY AND ARCH DIRECTIVES ================================================================ ;
TODO: org 0x0
TODO: bits 16

; ==== BOOTLOADER METHODS ======================================================================== ;
start:                                              ; Jump to main label
    jmp main

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

; ==== BOOTLOADER MAIN =========================================================================== ;
main:
    mov si, start_text
    call print

    cli                                             ; Disable interrupts: CPU can't exit of halt state
    hlt                                             ; Stop executing

; ==== CONSTANT DATA DIRECTIVES ================================================================== ;
start_text: db START_TEXT, LF, CR