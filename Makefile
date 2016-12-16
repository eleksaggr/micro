all: mb_header.asm boot.asm
	nasm -f elf64 mb_header.asm
	nasm -f elf64 boot.asm
	ld -n -o kernel.bin -T linker.ld mb_header.o boot.o

clean:
	rm kernel.bin *.o
