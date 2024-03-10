; ==== MACROS - ISRs DECLARATION ============================================================================= ;
; Declare ISR that pushes the interrupt number and jumps to the common ISR dispatcher.
; If the CPU doesn't push an error code for the interrupt, also push a dummy error code (0).
; The ISR is also made visible to the linker so that in can be used in the Rust module.
;! Using INT to trigger a handler that would normally be triggered by a CPU exception with an error
;! code will cause unexpected behaviour: there is no error code on the stack (nobody pushed it).
; https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
; "Intel® 64 and IA-32 Architectures Software Developer’s Manual: 3A"
; There are 3 types of CPU exceptions (Section 6.5):
; - Faults: CPU saves its state before the error - CS EIP point to faulting instruction.
; - Trap: CPU saves its state after the error - CS EIP point to next instruction (or pointed - i.e.: jumps).
; - Abort: No program restart, severe error.
; Some errors (such as div by 0) are faults: the handler has to fix or stop execution, or the error would loop.
; Section 6.15 describes the type of each of the CPU exceptions.
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
; No ring change saves:         EFLAGS, CS, EIP, ErrorCode? (for Exceptions 8, 10, 11, 12, 13, 14 17, 21)
; Ring change saves:   SS, ESP, EFLAGS, CS, EIP, ErrorCode?
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