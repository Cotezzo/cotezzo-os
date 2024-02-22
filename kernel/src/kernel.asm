; ==== CONSTANTS ============================================================================================= ;
%define LF 0x0D
%define CR 0x0A
%define START_TEXT `Kernel started!`

; ==== MEMORY AND ARCH DIRECTIVES ============================================================================ ;
org 0x10000
bits 32

; ==== BOOTLOADER MAIN ======================================================================================= ;
main:
    mov esi, text32p_test
    call print32p

    cli
    hlt

; ==== METHODS =============================================================================================== ;
%define VGA_TEXT_BUFFER 0xB8000
text32p_test: db START_TEXT, 0

print32p:
    [bits 32]
    push eax
    push esi                                         ; Store SI
    push edi

    mov edi, VGA_TEXT_BUFFER

    mov ah, 0x01                                    ; Forecolor (4b) & Foreground (4b) color: Black, Green

    .loop:                                          ; Loop start
        lodsb                                       ; Load content of ds:[si] in AL, inc SI
        test al, al                                 ; Check whether the character was NULL (last one)
        jz .end                                     ; If it is (Z flag is set since AL is 0), exit

        mov [edi], al                               ; Write loaded character to VGA buffer
        inc edi                                     ; Move pointer to next byte
        mov [edi], ah                               ; Write charcter colors to VGA buffer
        inc edi                                     ; Move pointer to next byte
        jmp .loop

    .end:

    pop edi
    pop esi                                          ; Restore previous SI value
    pop eax
    ret                                             ; If Z flag is set, return (pop ip)

; ==== CONSTANT DATA DIRECTIVES ============================================================================== ;
start_text: db START_TEXT, LF, CR