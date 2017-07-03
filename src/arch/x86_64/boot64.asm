global _start64
bits 64

extern kmain

_start64:
    call _enable_nx
    call _enable_wp

    call kmain

    cli
    mov rax, 0x4f724f204f534f4f
    mov [0xb8000], rax
    mov rax, 0x4f724f754f744f65
    mov [0xb8008], rax
    mov rax, 0x4f214f644f654f6e
    mov [0xb8010], rax
    hlt

_enable_nx:             ; Enable the No-Execute feature 
    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x800
    wrmsr
    ret

_enable_wp:             ; Enable the Write Protect feature 
    mov rcx, cr0
    or rcx, 0x10000
    mov cr0, rcx
    ret
