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
        mov eax, p4_table
        or eax, 0b11
        mov [p4_table + 511 * 8], eax

	mov eax, p3_table
	or eax, 0b11
	mov [p4_table], eax

	mov eax, p2_table
	or eax, 0b11
	mov [p3_table], eax

	mov ecx, 0
.map_p2_table:
	mov eax, 0x200000
	mul ecx
	or eax, 0b10000011
	mov [p2_table + ecx * 8], eax
	
	inc ecx
	cmp ecx, 512
	jne .map_p2_table

	ret

_enable_paging:
	mov eax, p4_table
	mov cr3, eax

	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	ret

_setup_SSE:
	mov eax, 0x1
	cpuid
	test edx, 1<<25
	jz .NoSSE

	; Enable SSE
	mov eax, cr0
	and ax, 0xFFFB
	or ax, 0x2
	mov cr0, eax
	mov eax, cr4
	or ax, 3 << 9
	mov cr4, eax

	ret
.NoSSE:
	mov al, "a"
	jmp _error

; The kernel entry point.
_start:
	; Setup the stack pointer.
	mov esp, stack_top 

	; Move Multiboot pointer to EDI
	mov edi, ebx

	call _check_multiboot
	call _check_cpuid
	call _check_long_mode

	call _setup_paging
	call _enable_paging

	call _setup_SSE

	lgdt [GDT64.Pointer]

	; Update selectors
	mov ax, 16
	mov ss, ax
	mov ds, ax
	mov es, ax

	jmp GDT64.Code:_start64

	; Throw error 0, if we could not jump to 64-bit entry point.
	mov al, "0"
	jmp _error

_error:
	mov dword[0xb8000], 0x0C520C45
	mov dword[0xb8004], 0x0C3A0C52
	mov dword[0xb8008], 0x0C200C20
	mov  byte[0xb800A], al
	hlt

section .rodata
; The Global Descriptor Table we use, once we are in 64-Bit mode.
GDT64:
	dq 0
	.Code equ $ - GDT64
	dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53)
	.Data equ $ - GDT64
	dq (1 << 44)  | (1<<47) | (1<<41)
	.Pointer:
	dw $ - GDT64 - 1
	dq GDT64

section .bss
align 4096
p4_table:
	resb 4096
p3_table:
	resb 4096
p2_table:
	resb 4096
stack_bottom:
	; Reserve 16KB for the stack.
	resb 4096 * 4
stack_top:
