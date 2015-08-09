.section .text
.globl flush_gdt
flush_gdt:
	ljmp $12*0x08, $complete_flush
complete_flush:
	mov $13*0x08, %ax // 3*0x08 is data segment
	mov %ax, %ss
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs
	ret

