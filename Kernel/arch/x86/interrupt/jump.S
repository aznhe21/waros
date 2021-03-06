.section .text
.globl flush_gdt
.globl task_switch
.globl task_leap

/* extern "C" fn flush_gdt(cs: u16, ds: u16) */
flush_gdt:
	pop %edx
	push $complete_flush
	/* (%esp)は飛び先、4(%esp)はcode segment */
	ljmp *(%esp)
complete_flush:
	/* data segmentをセット */
	mov 8(%esp), %ax
	mov %ax, %ss
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs
	mov %edx, (%esp)
	ret

/* extern "C" fn task_switch(csp: &mut *mut (), cip: &mut *mut (), nsp: *mut (), nip: *mut ())*/
task_switch:
	/* 全レジスタを保存(32+4バイト) */
	pusha
	pushf
	/* 飛び先をebpに入れる */
	mov 36+16(%esp), %ebp
	/* 現タスクのスタックを保存 */
	mov 36+4(%esp), %eax
	mov %esp, (%eax)
	/* 戻り先はrestore_stack */
	mov 36+8(%esp), %eax
	movl $restore_stack, (%eax)
	/* 新タスクのスタックを復元 */
	mov 36+12(%esp), %esp
	/* 新タスクにジャンプ */
	sti
	jmp *%ebp
restore_stack:
	/* 全レジスタを復元 */
	popf
	popa
	ret

/* extern "C" fn task_leap(sp: *mut (), ip: *mut ()) */
task_leap:
	/* 飛び先をebpに入れる */
	mov 8(%esp), %ebp
	/* 新タスクのスタックを復元 */
	mov 4(%esp), %esp
	sti
	jmp *%ebp

