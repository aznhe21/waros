.section .text
.globl idt_null_handler
.globl irq_handlers_begin
.globl irq_handlers_end
.extern idt_empty_handler
.extern irq_common_handler

idt_null_handler:
	pusha
	call idt_empty_handler
	popa
	iret

.macro define_irq_handler index
.globl irq_handler_\index
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

