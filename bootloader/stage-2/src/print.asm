
; ==== MEMORY AND ARCH DIRECTIVES ================ ;
bits 16

; ==== SECTIONS AND EXPORTS ====================== ;
; Make the following methods visible to the linker
global _print_char

; Define the following code in the .text section, so
; that we can control its location via linker script
section .text

; ==== METHODS =================================== ;
; Method impementing CDECL (C calling convetion):
; - Argument passed through stack from right to left
; - Caller removes parameters from stack
; - EAX ECX EDX: caller-saved; others: callee-saved
; - EAX stores returned value (int or pointer)
;
; Entering the method, assuming two input params,
; the stack looks like this:
; |  0x....  |  PC  |  P2  |  P1  | ...caller... |
;          sp --^
;
; Before returning, the SP must be restored to this
; state, oterwise the PC register would be "lost".
_print_char:
    push bp         ; Save previous BP state
    mov bp, sp      ; Store method stack "start"
                    ; Pushing stuff would change SP
    push bx         ; BX is not caller saved

    ; After pushing BP & BX, the last input param
    ; is at address BP plus:
    ; - sizeof PC (2B, Pushed by call instruction)
    ; - sizeof BP (2B, pushed by us)
    ; - sizeof BX (2B, pushed by us)
    ; Segment is not pushed, this is a near call.
    ;! For some reason (probably being Rust not
    ;! fully supporting 16b mode or some errors
    ;! in the custom target), the u8 parameters
    ;! pushed by Rust are threated as 32b values;
    ;! the stack pointer has to be moved by 4.
    mov al, [bp+6]  ; 1^ Rust param: to print char
    mov bh, [bp+10] ; 2^ Rust param: page number
    mov ah, 0x0E
    int 0x10        ; INT 10,E: TTY Output Char

    pop bx          ; Restore BX
    mov sp, bp      ; Restore SP
    pop bp          ; Restore BP

    retn            ; Return from near call