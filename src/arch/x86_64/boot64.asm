global _start64
bits 64

_start64:
	cli
	mov rax,  0x2f592f412f4b2f4f
	mov qword[0xb8000], rax
	hlt
