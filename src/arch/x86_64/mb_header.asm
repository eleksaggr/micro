section .multiboot_header
header_start:
	dd 0xe85250d6														; Magic Number (Multiboot 2)
	dd 0																; Architecture (x86)
	dd header_end - header_start										; Header length
	dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))		; Checksum
	dw 0																; Type
	dw 0																; Flags
	dd 8																; Size
header_end:
