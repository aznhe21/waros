.section .text
.globl swi_entry
.globl unused_entry
.globl irq_entry
.globl fiq_entry
.extern irq_handler
.extern unhandled_interrupt

swi_entry:
unused_entry:
fiq_entry:
	bl unhandled_interrupt
	b .

irq_entry:
	sub   lr, lr, #4;

	srsdb #0x1F!
	cpsid if, #0x1F
	push  {r0-r5, r12, lr}

	and   r4, sp, #4
	sub   sp, sp, r4

	bl    irq_handler

	add   sp, sp, r4

	pop   {r0-r5, r12, lr}
	rfeia sp!

