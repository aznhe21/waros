.section .text
.globl memset
.globl memory_fill8
.globl memory_fill16
.globl memory_fill32
.globl memory_fill64

memset:
	cmp	r2, #8
	blo	memset_tail		// jump if count < 8

	// val = val | val << 8 | val << 16 | val << 24
	and	r1, r1, #0xFF
	orr	r1, r1, LSL #8
	orr	r1, r1, LSL #16

	ands	r3, r0, #3		// shift = align(ptr, 4)
	beq	memset_main		// jump if ptr is aligned to 4 byte
	rsb	r3, r3, #4		// shift = 4 - shift
	sub	r2, r2, r3		// count -= shift
memset_under4:
	subs	r3, r3, #1		// shift -= 1
	strb	r1, [r0], #1
	bne	memset_under4		// loop while shift > 0

memset_main:
	movs	r3, r2, LSR #5		// len = count >> 4
	and	r2, r2, #15		// count &= 15
	beq	memset_tail		// jump if len == 0

	push	{r4-r7}
	mov	r4, r1
	mov	r5, r1
	mov	r6, r1
	mov	r7, r1
memset_fill:
	subs	r3, r3, #1		// len -= 1
	stmia	r0!, {r4-r7}
	bne	memset_fill		// loop while len > 0

	pop {r4-r7}

memset_tail:
	cmp	r2, #0
	moveq	pc, lr			// return if count == 0
memset_over:
	subs	r2, r2, #1		// count -= 1
	strb	r1, [r0], #1
	bne	memset_over		// while count > 0

	mov pc, lr

memory_fill8:
	cmp	r2, #16
	blo	memory_fill8_lt16	// jump if count < 16

	push	{r4-r7}

	// val = val | val << 8 | val << 16 | val << 24
	and	r1, r1, #0xFF
	orr	r1, r1, LSL #8
	orr	r1, r1, LSL #16

	mov	r4, r1
	mov	r5, r1
	mov	r6, r1
	mov	r7, r1

	add	r2, r0, r2		// last = ptr + count

	b	2f

1:	stmia	r0!, {r4-r7}

2:
	cmp	r0, r2
	blo	1b			// while ptr < last

	pop	{r4-r7}
	mov	pc, lr

memory_fill8_lt16:
	cmp	r2, #0
	moveq	pc, lr			// return if count == 0

memory_fill8_loop:
	subs	r2, r2, #1
	strb	r1, [r0], #1
	bne	memory_fill8_loop	// while count != 0
	mov	pc, lr

memory_fill16:
	cmp	r2, #8
	blo	memory_fill16_lt8	// jump if count < 8

	push	{r4-r7}

	// val = val | val << 16
	mov	r1, r1, LSL #16
	orr	r1, r1, LSR #16

	mov	r4, r1
	mov	r5, r1
	mov	r6, r1
	mov	r7, r1

	add	r2, r0, r2, LSL #1	// last = ptr + count * 2

	b	2f

1:	stmia	r0!, {r4-r7}

2:
	cmp	r0, r2
	blo	1b			// while ptr < last

	pop	{r4-r7}
	mov	pc, lr

memory_fill16_lt8:
	cmp	r2, #0
	moveq	pc, lr			// return if count == 0

memory_fill16_loop:
	subs	r2, r2, #1
	strh	r1, [r0], #2
	bne	memory_fill16_loop	// while count != 0
	mov	pc, lr

memory_fill32:
	cmp	r2, #4
	blo	memory_fill32_lt4	// jump if count < 4

	push	{r4-r7}

	mov	r4, r1
	mov	r5, r1
	mov	r6, r1
	mov	r7, r1

	add	r2, r0, r2, LSL #2	// last = ptr + count * 4

	b	2f

1:	stmia	r0!, {r4-r7}

2:
	cmp	r0, r2
	blo	1b			// while ptr < last

	pop	{r4-r7}
	mov	pc, lr

memory_fill32_lt4:
	cmp	r2, #0
	moveq	pc, lr			// return if count == 0

memory_fill32_loop:
	subs	r2, r2, #1
	str	r1, [r0], #4
	bne	memory_fill32_loop	// while count != 0
	mov	pc, lr

memory_fill64:
	ldr	r12, [sp]
	cmp	r12, #1
	stmeq	r0, {r2-r3}		// store if count == 1
	movls	pc, lr			// return if count <= 1

	push	{r4-r5}

	mov	r4, r2
	mov	r5, r3

	add	r12, r0, r12, LSL #3	// last = ptr + count * 8

	b	2f

1:	stmia	r0!, {r2-r5}

2:
	cmp	r0, r12
	blo	1b			// while ptr < last

	pop	{r4-r5}
	mov	pc, lr

