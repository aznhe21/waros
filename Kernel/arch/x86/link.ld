ENTRY(start)
OUTPUT_FORMAT(elf32-i386)

KERNEL_BASE = 0xC0000000;

SECTIONS {
	. = 0x100000;
	. += SIZEOF_HEADERS;

	.init : AT(ADDR(.init)) {
		KEEP( *(.multiboot) )
		*(.inittext)
	}

	. += KERNEL_BASE;

	__kernel_start = . - SIZEOF(.init) - SIZEOF_HEADERS;

	.text ALIGN(0x1000) : AT(ADDR(.text) - KERNEL_BASE) {
		*(.text .text.*)
	}

	/* read-only data, page aligned to allow use of the no-execute feature */
	.rodata ALIGN(0x1000) : AT(ADDR(.rodata) - KERNEL_BASE) {
		*(.rodata .rodata.*)
	}

	/* Read-write data, page aligned for the .padata section */
	.data ALIGN(0x1000) : AT(ADDR(.data) - KERNEL_BASE) {
		*(.data .data.*)
	}

	/* Zero-initialised data */
	.bss ALIGN(0x1000) : AT(ADDR(.bss) - KERNEL_BASE) {
		__kernel_bss_start = .;
		*(.bss .bss.*)
		__kernel_bss_end = .;
	}

	__kernel_end = .;
}
