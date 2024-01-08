; ==== MEMORY AND ARCH DIRECTIVES ================ ;
bits 16

; ==== SECTIONS AND EXPORTS ====================== ;
; Make the following methods visible to the linker
global _value_print_test
global _pointer_print_test

; Define the following code in the .text section, so
; that we can control its location via linker script
section .text

; ==== METHODS =================================== ;
_value_print_test:
    push bp
    mov bp, sp
    push bx

    mov al, [bp+6]  ; 1^ Rust param: to print char
    mov bh, [bp+10] ; 2^ Rust param: page number
    mov ah, 0x0E
    int 0x10        ; INT 10,E: TTY Output Char

    pop bx
    mov sp, bp
    pop bp
    retn

_pointer_print_test:
    push bp
    mov bp, sp
    push si
    push bx

    ; [bp+6] contains the first parameter, which is
    ; a pointer to a u8 variable.
    ; Load the pointer value to si, then access the
    ; value stored in [si] and save it to AL.
    ; AL has the same value of the original u8 var.
    ; Print AL value.
    mov si, [bp+6]
    mov al, [si]
    mov bh, 0
    mov ah, 0x0E
    int 0x10

    pop bx
    pop si
    mov sp, bp
    pop bp
    retn