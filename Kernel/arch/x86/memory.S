.section .text
.globl memset
.globl memory_fill8
.globl memory_fill16
.globl memory_fill32
.globl memory_fill64

memset:
	push	%edi
	cld
	mov	0x08(%esp), %edi
	mov	0x0C(%esp), %eax
	mov	0x10(%esp), %ecx
	rep	stosb
	pop	%edi
	mov	0x04(%esp), %eax
	ret

memory_fill8:
	push	%edi
	cld
	mov	0x08(%esp), %edi
	mov	0x0C(%esp), %eax
	mov	0x10(%esp), %ecx
	rep	stosb
	pop	%edi
	ret

memory_fill16:
	push	%edi
	cld
	mov	0x08(%esp), %edi
	mov	0x0C(%esp), %eax
	mov	0x10(%esp), %ecx
	rep	stosw
	pop	%edi
	ret

memory_fill32:
	push	%edi
	cld
	mov	0x08(%esp), %edi
	mov	0x0C(%esp), %eax
	mov	0x10(%esp), %ecx
	rep	stosl
	pop	%edi
	ret

memory_fill64:
	ret

