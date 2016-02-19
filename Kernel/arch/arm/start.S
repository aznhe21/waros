.section .inittext
.globl _start
.extern atags_ptr
_start:
	/* atagsを保存 */
	ldr   r3, =atags_ptr
	str   r2, [r3]

	b     reset_entry

.extern undefined_entry
.extern swi_entry
.extern prefetch_entry
.extern data_entry
.extern unused_entry
.extern irq_entry
.extern fiq_entry
_vectors:
	ldr pc, reset_code
	ldr pc, undefined_code
	ldr pc, swi_code
	ldr pc, prefetch_code
	ldr pc, data_code
	ldr pc, unused_code
	ldr pc, irq_code
	ldr pc, fiq_code

reset_code:     .word reset_entry
undefined_code: .word undefined_entry
swi_code:       .word swi_entry
prefetch_code:  .word prefetch_entry
data_code:      .word data_entry
unused_code:    .word unused_entry
irq_code:       .word irq_entry
fiq_code:       .word fiq_entry

.extern arm_main
.extern kmain
reset_entry:
	cpsid aif, #0x1F

	/* ベクタテーブルを転送 */
	ldr   r3, =_vectors
	mov   r4, #0x0000
	ldmia r3!, {r5-r12}
	stmia r4!, {r5-r12}
	ldmia r3!, {r5-r12}
	stmia r4!, {r5-r12}

	/* IRQスタック初期化 */
	msr   cpsr_c, #0xD2
	ldr   sp, =__irq_stack

	/* FIQスタック初期化 */
	msr   cpsr_c, #0xD1
	ldr   sp, =__fiq_stack

	/* 割り込み無効化、スタック初期化 */
	msr   cpsr_c, #0xDF
	ldr   sp, =__usr_stack

	/* 非アラインアクセスを許可 */
	mrc   p15, 0, r4, c1, c0, 0
	orr   r4, #1 << 22
	mcr   p15, 0, r4, c1, c0, 0

	/* TLB、命令キャッシュ、データキャッシュ、分岐予測キャッシュを初期化 */
	mov   r4, #0
	mcr   p15, 0, r4, c8, c7, 0
	mcr   p15, 0, r4, c7, c7, 0
	mcr   p15, 0, r4, c7, c6, 0
	mcr   p15, 0, r4, c7, c5, 6
	mcr   p15, 0, r4, c7, c10, 4 // ライトバッファをドレイン

	/* BSSをクリア */
	ldr   r4, =__kernel_bss_start
	ldr   r9, =__kernel_bss_end
	mov   r5, #0
	mov   r6, #0
	mov   r7, #0
	mov   r8, #0
	b     2f

1:
	stmia r4!, {r5-r8}

2:
	cmp   r4, r9
	blo   1b

	/* エントリポイントを呼び出す */
	ldr   r4, =arm_main
	blx   r4

	ldr   r4, =kmain
	blx   r4

	/* If kmain returns, loop forever */
.l:
	wfe
	b     .l

/* === Read-write data === */
.section .data
.globl atags_ptr
atags_ptr:
	.long -1
.section .bss

