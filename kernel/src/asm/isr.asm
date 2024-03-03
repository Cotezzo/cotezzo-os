; ==== MACROS - ISRs DECLARATION ============================================================================= ;
; Declare ISR that pushes the interrupt number and jumps to the common ISR dispatcher.
; If the CPU doesn't push an error code for the interrupt, also push a dummy error code (0).
; The ISR is also made visible to the linker so that in can be used in the Rust module.
%macro isr 1
    global _c_isr_%1

    _c_isr_%1:
        %if %1!=8 && %1!=10 && %1!=11 && %1!=12 && %1!=13 && %1!=14 && %1!=17 && %1!=21
        push 0
        %endif
        push %1
        jmp _c_isr_dispatcher
%endmacro

; Run ISR declaration macro for each of the 256 possible interrupts
%assign i 0
%rep 256
    isr i
%assign i i+1
%endrep

; ==== ISRs DISPATCHER ======================================================================================= ;
; Rust ISR dispatcher implementation
extern _rs_isr_dispatcher

; Before calling ISR, CPU pushes some registers values to save its state.
; If ISR is called by lower privilege-level, the stack used is switched; stack informations are also pushed.
; No ring change saves registers:   EFLAGS, CS, EIP, ErrorCode? (for Exceptions 8, 10, 11, 12, 13, 14 17, 21)
; Ring change saves registers:     SS, ESP, EFLAGS, CS, EIP, ErrorCode?
_c_isr_dispatcher:
    ; General use registers may be needed in case of exception or interrupt for further analysis
    ; Pushes (in order): EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI
    ; Since we're using the C decl for the inner handling, this also takes care of pushing EAX, ECX, EDX.
    pusha

    ; Setup data segment as if the ring switch had occurred (setup kernel data segments)
    push ds                                         ; Save previous DS value
    xor eax, eax                                    ; 0 EAX register
    mov eax, 0x10                                   ; TODO: extern Data Segment Offset from gdt.rs
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov fs, ax

    ; Pass ESP as argument - to be used as a pointer to the stack info:
    ; Interrupt number, error code, CPU saved state, general purpose registers, DS)
    push esp

    ; Call Rust ISR and remove arguments from stack
    call _rs_isr_dispatcher
    add esp, 4

    ; Restore data segments and general registers
    pop eax ; pop ds
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov fs, ax
    popa

    ; Remove arguments pushed from specific ISR (interrupt number, error code)
    add esp, 8

    ; Special return from interrupt instruction - Restores the previously saved registers before returning
    iret