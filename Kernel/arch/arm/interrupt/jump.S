.section .text
.globl task_entry
.globl task_switch
.globl task_leap
.globl task_switch_sp

task_entry:
	mov r0, r4
	mov lr, r5
	mov pc, r6

/* task_switch(cur_regs, next_regs) */
task_switch:
	mrs   r12, cpsr
	stmia r0!, {r4 - r14}
	mov   r0, r1

/* task_leap(regs) */
task_leap:
	add   r0, r0, #4*11
	ldmdb r0!, {r4 - r14}
	msr   cpsr_cxsf, r2
	mov   pc, lr

