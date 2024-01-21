
; ==== MEMORY AND ARCH DIRECTIVES ============================================================================ ;
; Code to be used in a 32bp mode environment.
bits 32

; ==== REAL MODE / PROTECTED MODE MACROS ===================================================================== ;
; Define macro name and number of parameters
; These blocks cannot be declared in actual
; methods since they deal with real/protected
; modes and segments.

;* Switches from 32pm to 16rm. Sets the segments
;* value to 0, keeps offsets as-is.
%macro from_32pm_to_16rm 0
    cli                                             ; Disable interrupts for switching
    push eax
    
    jmp word 0x18:.16pm                             ; Jump to 16b segment (word -> address size)

    .16pm:
    [bits 16]

    mov eax, cr0                                    ; Read control register 0 value
    and al, ~1                                      ; Unset first bit to switch back to real mode
    mov cr0, eax                                    ; Update CR0 value

    xor ax, ax                                      ; Setup segment registers to 0, GDT is not used now
    mov ds, ax
    mov es, ax
    mov ss, ax
    jmp word 0x00:.16rm                             ; Setup CS register for real mode

    .16rm:
    pop eax
    sti                                             ; Mode switch completed, re-enable interrupts
%endmacro

;* Switches from 16rm to 32pm. Does not load GDT
;* since it must be called after switching back to
;* 16rm, not as the first time swithing.
;* The segment selectors are hardcoded and refer to
;* the selectors defined in main.asm.
; TODO: find a way to make 'extern gdt' work
; TODO:     and avoid hardcoding these values.
%macro from_16rm_to_32pm 0
    cli                                             ; Disable interrupts for switching
    push eax
    
    ; GDT descriptor already loaded

    mov eax, cr0                                    ; Read control register 0 value
    or al, 1                                        ; Set first bit (protected mode)
    mov cr0, eax                                    ; Update CR0 value

    jmp dword 0x08:.32pm                            ; Setup Code Segment selector

    .32pm:
    [bits 32]
    mov ax, 0x10                                    ; Setup Data Segment selector
    mov ds, ax
    mov ss, ax
    mov es, ax

    pop eax
    ;! Re-enabling interrupts eventually crashes
    ;sti                                            ; Mode switch completed, re-enable interrupts
%endmacro


;* Procudes a segmented memory address from the
;* given linear address.
;* Input:
;* - Linear Memory Address
;* - Output Segment Register
;* - Output offset register (32bit version)
;* - Output offset register (16bit version)
%macro linear_to_segmented 4
    mov %3, %1                                      ; Store linear address in 32b register
    shr %3, 4                                       ; Only keep top 16 bits
    mov %2, %4                                      ; Store shifted addr (segment) in segment register
    
    mov %3, %1                                      ; Store linear address in 32b register
    and %3, 0xF                                     ; Only keep low 4 bits - Offset is already in the right reg
%endmacro


; ==== GLOBALS AND EXTERN METHODS ============================================================================ ;
; Make the following methods visible to the linker.
global _c_disk_reset
global _c_disk_read
global _c_disk_get_params

; ==== CODE SECTION ========================================================================================== ;
; Define the following code in the .text section, so that we can control its location with linker script.
section .text

;? CDECL (C calling convention):
; - Argument passed through stack from right to left
; - Caller removes parameters from stack
; - EAX ECX EDX: caller-saved; others: callee-saved
; - EAX stores returned value (int or pointer)
; Entering the method, the stack looks like this:
; | .. | EPC | param1 | param2 | ..caller stack.. |
;         esp --^
; Before returning, the SP must be restored to this
; state, otherwise the EPC register would be "lost".

;* Resets the disk controller (to use in case of
;* I/O errors).
;* Input parameters (from last pushed / left):
;* - Drive number
;* Output:
;* - Outcome of the operation (1 success, 0 error)
; This method implements the C calling convention.
_c_disk_reset:
    [bits 32]
    push ebp                                         ; Save previous BP state
    mov ebp, esp                                      ; Store method stack "start" - Pushing stuff would change SP
    ; No need to push DX, is caller saved

    ; After pushing EBP, input parameters start at
    ; address EBP plus 8:
    ; - sizeof EPC (4B, pushed by call instruction)
    ; - sizeof EBP (4B, pushed by us)
    ; Segment is not pushed, this is a near call.
    mov dl, [ebp+8]                                  ; 1^ Rust param: to reset drive

    ; Return to real mode
    from_32pm_to_16rm

    stc                                             ; Reset CF to 1 to read the outcome of INT
    mov ah, 0x00                                    ; INT 13, 0: Reset Disk Controller
    int 0x13

    ; Return to protected mode
    from_16rm_to_32pm

    mov eax, 1                                       ; AX is the return value, it should reflect INT CF state
    sbb eax, 0                                       ; ax = ax - (0 + CF) (CF 0 -> AX 1, CF 1 -> AX 0)

    mov esp, ebp                                      ; Restore SP
    pop ebp                                          ; Restore BP

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
_c_disk_read:
    [bits 32]

    ; Setup and save stack pointers
    push ebp
    mov ebp, esp
    push ebx                                        ; BX is not caller saved
    push es                                         ; ES is not caller saved

    ; Parameters read and setup
    mov dl, [ebp+8]                                  ; 1^ Rust param: to read drive, already set up for INT
    mov ax, [ebp+12]                                 ; 2^ Rust param: target cylinder
    mov dh, [ebp+16]                                 ; 3^ Rust param: target head, already set up for INT
    mov cl, [ebp+20]                                 ; 4^ Rust param: target sector, already set up for INT

    mov ch, al                                      ; INT expects lower 8b of Cylinder value in CH
    shl ah, 6                                       ; INT expects upper 2b of Cylinder value in last CL's bits
    or cl, ah                                       ; OR to keep CL's lower 6 to previous value (target sector)
                                                    ; CX        = ---CH--- ---CL---
                                                    ; Cylinder  = 76543210 98
                                                    ; Sector    =            543210

    mov al, [ebp+24]                                ; 5^ Rust param: sectors to read, already set up for INT
    
    ;mov bx, [ebp+28]                               ; 6^ Rust param: loading address, already set up for INT
    ;mov es, bx                                     ; Far pointer, segment and offset are both pushed
    ;mov bx, [ebp+32]

    ; Return to real mode
    from_32pm_to_16rm
    
    linear_to_segmented [ebp+28], es, ebx, bx       ; 6^ Rust param: loading address, already set up for INT
                                                    ; Convert linear address to segmented memory model address
    ;> mov byte [es:bx], 'c'                        ; TEST: put something at pointed memory
    stc                                             ; Reset CF to 1 to read the outcome of INT
    mov ah, 0x02
    int 0x13                                        ; INT 13, 2: Read Disk Sectors

    ; Return to protected mode
    from_16rm_to_32pm

    mov eax, 1                                       ; AX is the return value, it should reflect INT CF state
    sbb eax, 0                                       ; ax = ax - (0 + CF) (CF 0 -> AX 1, CF 1 -> AX 0)
    
    ; Restore registers and return
    pop es
    pop ebx
    mov esp, ebp
    pop ebp
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
_c_disk_get_params:
    [bits 32]

    push ebp
    mov ebp, esp
    push ebx                                        ; BX is not caller saved
    push esi                                        ; SI is not caller saved
    push edi                                        ; DI is not caller saved
    push es                                         ; ES is not caller saved

    mov dl, [ebp+8]                                 ; 1^ Rust param: to read drive

    ; Return to real mode
    from_32pm_to_16rm

    stc                                             ; Reset CF to 1 to read the outcome of INT
    xor di, di                                      ; INT 13, 8 requires ES:DI to be 0000:0000
    mov es, di
    mov ah, 0x08                                    ; INT 13, 8: Read Disk Parameters
    int 0x13

    ; Return to protected mode
    from_16rm_to_32pm

    pop es
    pop edi

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
    ; DH = maximum head number (0 based)
    ; DL = number of drives
    ; ES:DI -> drive parameter table (floppies only)

    jc .exit                                        ; If CF is set, error: jump to end without preparing output

    mov esi, [ebp+12]                               ; 2^ Rust param: drive type output address
    mov byte [esi], bl

    mov bl, ch
    mov bh, cl
    shr bh, 6
    inc bx                                          ; Cyinders number is 0 based, + 1 for real number
    mov esi, [ebp+16]                               ; 3^ Rust param: cylinders count output address
    mov word [esi], bx

    inc dh                                          ; Heads number is 0 based, + 1 for real number
    mov esi, [ebp+20]                               ; 4^ Rust param: heads count output address
    mov byte [esi], dh

    and cl, 0x3F
    mov esi, [ebp+24]                               ; 5^ Rust param: sectors count output address
    mov byte [esi], cl

    .exit:
    mov eax, 1                                      ; AX is the return value, it should reflect INT CF state
    sbb eax, 0                                      ; ax = ax - (0 + CF) (CF 0 -> AX 1, CF 1 -> AX 0)

    pop esi
    pop ebx
    mov esp, ebp
    pop ebp
    retn