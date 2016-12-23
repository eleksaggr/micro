global _start
bits 32

extern _start64

section .text

; Check if the bootloader is Multiboot compliant. This will throw an error with code 1, if not compliant.
_check_multiboot:
	; Compare with the bootloader supplied magic number.
	cmp eax, 0x36d76289
	jne .NoMultiboot
	ret
.NoMultiboot:
	mov al, "1"
	jmp _error

; Check if the CPU supports CPUID. Throws error 2, if not supported.
_check_cpuid:
	; Push FLAGS to the stack
	pushfd 
	pop eax

	mov ecx, eax

	; Flip the ID bit (Bit 21)
	xor eax, 1 << 21 

	; Copy EAX to FLAGS
	push eax
	popfd

	; Copy FLAGS to EAX	
	pushfd;
	pop eax;

	; Restore FLAGS
	push ecx
	popfd

	; Compare EAX and ECX, to see if CPUID is supported.
	xor eax, ecx
	jz _check_cpuid.NoCPUID
	ret	
.NoCPUID:
	mov al, "2"
	jmp _error

; Check if long mode is supported. Throws error 3, if not supported.
_check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .NoLongMode

	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29
	jz .NoLongMode
	ret
.NoLongMode:
	mov al, "3"
	jmp _error

_setup_paging:
; Clear the page tables
._clear_tables:
	; Start PDPT at 0x1000
	mov edi, 0x1000
	mov cr3, edi
	xor eax, eax
	mov ecx, 4096
	rep stosd
	mov edi, cr3
	; Link the pages tables
._setup_tables:
	mov dword[edi], 0x2003
	add edi, 0x1000
	mov dword[edi], 0x3003
	add edi, 0x1000
	mov dword[edi], 0x4003
	add edi, 0x1000
._id_map:
	mov ebx, 0x00000003
	mov ecx, 512
._set_entry:
	mov dword[edi], ebx
	add ebx, 0x1000
	add edi, 8
	loop _setup_paging._set_entry
._enable_pae:
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax
	ret

_enable_long_mode:
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax
	ret

GDT64:
	.Null: equ $ - GDT64
	dw 0
	dw 0
	db 0
	db 0
	db 0
	db 0
	.Code: equ $ - GDT64
	dw 0
	dw 0
	db 0
	db 0x9A
	db 0x20
	db 0
	.Data: equ $ - GDT64
	dw 0
	dw 0
	db 0
	db 0x92
	db 0x00
	db 0
	.Pointer:
	dw $ - GDT64 - 1
	dq GDT64

; The kernel entry point.
_start:
	; Setup the stack pointer.
	mov esp, stack_top 

	call _check_multiboot
	call _check_cpuid
	call _check_long_mode

	call _setup_paging
	call _enable_long_mode
	
	lgdt [GDT64.Pointer]
	jmp GDT64.Code:_start64

	mov al, "0"
	jmp _error

_error:
	mov dword[0xb8000], 0x0C520C45
	mov dword[0xb8004], 0x0C3A0C52
	mov dword[0xb8008], 0x0C200C20
	mov  byte[0xb800A], al
	hlt

section .bss
align 16
stack_bottom:
	; Reserve 16KB for the stack.
	resw 16384
stack_top:
