/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arcm/x86/start.S
 * - x86 Entrypoint
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/* The kernel is linked to run at 3GB */
LINKED_BASE = 0xC0000000

/* === Multiboot Header === */
MULTIBOOT_PAGE_ALIGN  =  (1<<0)
MULTIBOOT_MEMORY_INFO =  (1<<1)
MULTIBOOT_REQVIDMODE  =  (1<<2)
MULTIBOOT_HEADER_MAGIC =  0x1BADB002
MULTIBOOT_HEADER_FLAGS = (MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO | MULTIBOOT_REQVIDMODE)
MULTIBOOT_CHECKSUM     = -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
.section .multiboot, "a"
.globl mboot
mboot:
	.long MULTIBOOT_HEADER_MAGIC
	.long MULTIBOOT_HEADER_FLAGS
	.long MULTIBOOT_CHECKSUM
	.long mboot
	/* a.out kludge (not used, the kernel is elf) */
	.long 0, 0, 0, 0	/* load_addr, load_end_addr, bss_end_addr, entry_addr */
	/* Video mode */
	.long 0 	/* Mode type (0: LFB) */
	.long 0 	/* Width (no preference) */
	.long 0 	/* Height (no preference) */
	.long 32	/* Depth (32-bit preferred) */

.extern x86_prep_page_table
/* === Code === */
.section .inittext, "ax"
.globl start
start:
	/* EFLAGSをリセット */
	pushl $0
	popf

	/* Multibootのアドレスを保存 */
	movl %eax, mboot_sig - LINKED_BASE
	movl %ebx, mboot_ptr - LINKED_BASE

	movl $0, %gs:0x30 /* __morestackで必要 */

	/* BSSをクリア */
	/* memory::fill32(&__kernel_bss_start, 0, (&__kernel_bss_end - &__kernel_bss_start) / 4) */
	cld
	xor %eax, %eax
	mov $__kernel_bss_start - LINKED_BASE, %edi
	mov $__kernel_bss_end - LINKED_BASE, %ecx
	sub %edi, %ecx
	shr $2, %ecx
	rep stosl

	/* ページテーブルの準備 */
	mov $init_stack - LINKED_BASE, %esp
	push $init_pt - LINKED_BASE
	call x86_prep_page_table - LINKED_BASE
	add 4, %esp

	/* ページングを有効化 */
	mov $init_pd - LINKED_BASE, %eax
	mov %eax, %cr3

	/* 4MBのページングを有効化 */
	/*mov %cr4, %eax
	or  $0x00000010, %eax
	mov %eax, %cr4*/

	/* ページング及び書き込み保護 */
	mov %cr0, %eax
	or $0x80010000, %eax
	mov %eax, %cr0

	mov $init_stack, %esp
	call x86_pre_init

	/* higherにジャンプしてCSを2*0x08（コードセグメント）にセット */
	ljmp $2*0x08, $start_high
.section .text
.extern kmain
start_high:
	/* Clear identity mapping */
	movl $0, init_pd+0
	invlpg 0

	/* Prep segment registers */
	mov $3*0x08, %ax // 3*0x08 is data segment
	mov %ax, %ss
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs

	call kmain

	/* If kmain returns, loop forever */
.l:
	hlt
	jmp .l

/* === Read-write data === */
.section .data
init_pd:
	.long init_pt - LINKED_BASE + 3
	.rept 768-1
		.long 0
	.endr
	.long init_pt - LINKED_BASE + 3
	.long init_pt2 - LINKED_BASE + 3
	.rept 255-1
		.long 0
	.endr
init_pt:
	/* The contents of this table is filled by the x86_prep_page_table function */
	.rept 1024
		.long 0
	.endr
init_pt2:
	.rept 1024
		.long 0
	.endr
.globl mboot_sig
.globl mboot_ptr
mboot_sig:
	.long 0
mboot_ptr:
	.long 0

.section .bss
	.space 0x1000*2
init_stack:

