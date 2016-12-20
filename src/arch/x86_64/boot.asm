section .bss
align 16
stack_bottom:
	; Reserve 16KB for the stack.
	resw 16384
stack_top:

section .text
; The kernel entry point.
global _start
bits 32
_start:
	mov esp, stack_top ; Setup the stack pointer.

	; Print "OK" to the screen.
	mov al, "1"
	jmp error
	hlt

error:
	mov dword[0xb8000], 0x0C520C45
	mov dword[0xb8004], 0x0C3A0C52
	mov dword[0xb8008], 0x0C200C20
	mov  byte[0xb800A], al
	hlt
