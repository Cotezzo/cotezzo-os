; Expose LGDT method to the linker
global _c_load_gdt

;* Calls the ASM 'lgdt' metho, loading the GDT using
;* the provided GDT Descriptor in input.
;* Input parameters (from last pushed / left):
;* - GDT Descriptor Address
;* - GDT entry address to be loaded as Code Segment
;* - GDT entry address to be loaded as Data Segment
; This method implements the C calling convention.
_c_load_gdt:
    [bits 32]

    ; Save previous BP, save base pointer
    push ebp
    mov ebp, esp

    ; Call LGDT with input descriptor address
    mov eax, [ebp + 8]
    lgdt [eax]

    ; ECX -> Code Segment entry offset to be loaded
    mov ecx, [ebp + 12]

    ; CS can't be directly set, call far return to
    ; pop ECX and _ address to CS an PC.
    push ecx
    push ._
    retf
    ._:

    ; EDX -> Data Segment entry offset to be loaded
    mov edx, [ebp + 16]

    mov ds, dx
    mov es, dx
    mov ss, dx
    mov fs, dx
    mov gs, dx

    ; Restore stack pointers and return
    mov esp, ebp
    pop ebp
    retn