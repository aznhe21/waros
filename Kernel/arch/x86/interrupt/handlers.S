.section .text
.globl idt_null_handler
.globl idt_0c_handler
.globl idt_0d_handler
.globl idt_0e_handler
.extern idt_empty_handler
.extern page_fault_handler
.extern general_protection_fault_handler
.extern stack_segment_fault_handler

idt_null_handler:
	pusha
	call idt_empty_handler
	popa
	iret

idt_0c_handler:
	pusha
	push 32(%esp)
	call stack_segment_fault_handler
	add $4, %esp
	popa
	iret

idt_0d_handler:
	pusha
	push 32(%esp)
	call general_protection_fault_handler
	add $4, %esp
	popa
	iret

idt_0e_handler:
	pusha
	mov %cr2, %eax
	push %eax
	mov %esp, %eax
	add $(32+4), %eax
	push %eax
	call page_fault_handler
	add $8, %esp
	popa
	iret

.macro define_irq_handler index
.globl irq_handler_\index
.extern irq_common_handler

irq_handler_\index:
	pusha
	pushl $\index
	call irq_common_handler
	addl $4, %esp
	popa
	iret
.endm

	define_irq_handler 0
	define_irq_handler 1
	define_irq_handler 2
	define_irq_handler 3
	define_irq_handler 4
	define_irq_handler 5
	define_irq_handler 6
	define_irq_handler 7
	define_irq_handler 8
	define_irq_handler 9
	define_irq_handler 10
	define_irq_handler 11
	define_irq_handler 12
	define_irq_handler 13
	define_irq_handler 14
	define_irq_handler 15

