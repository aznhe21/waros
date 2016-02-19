OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")
OUTPUT_ARCH(arm)
ENTRY(_start)

KERNEL_BASE = DEFINED(MACH_bcm) ? 0x8000 : 0x10000;

IRQ_STACK_SIZE = 2 * 1024;
FIQ_STACK_SIZE = 2 * 1024;
ABT_STACK_SIZE = 0;
UND_STACK_SIZE = 0;
USR_STACK_SIZE = 8 * 1024;

SECTIONS {
	. = KERNEL_BASE;

	__kernel_start = .;

	.text : {
		KEEP(*(.inittext))
		*(.text .text.*)
	}

	/* read-only data, page aligned to allow use of the no-execute feature */
	.rodata : {
		*(.rodata .rodata.*)
		. = ALIGN(4);
	}

	/* Read-write data, page aligned for the .padata section */
	.data : {
		*(.data .data.*)
		. = ALIGN(4);
	}

	/* Zero-initialised data */
	.bss : {
		__kernel_bss_start = .;
		*(.bss .bss.*)
		. = ALIGN(4);
		__kernel_bss_end = .;
	}

	/* Stack */
	.stack : {
		. += IRQ_STACK_SIZE;
		. = ALIGN(4);
		__irq_stack = .;

		. += FIQ_STACK_SIZE;
		. = ALIGN(4);
		__fiq_stack = .;

		. += ABT_STACK_SIZE;
		. = ALIGN(4);
		__abt_stack = .;

		. += UND_STACK_SIZE;
		. = ALIGN(4);
		__und_stack = .;

		. += USR_STACK_SIZE;
		. = ALIGN(4);
		__usr_stack = .;
	}

	__kernel_end = .;
}
