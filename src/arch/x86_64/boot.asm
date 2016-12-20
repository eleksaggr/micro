section .bss
align 16
stack_bottom:
resw 16384 ; Reserve 16KB for the stack.
stack_top:

section .text
; The kernel entry point.
global _start

bits 32
_start:
	mov esp, stack_top ; Setup the stack pointer.

	; Print "OK" to the screen.
	mov dword[0xb8000], 0x2f4b2f4f
	hlt
