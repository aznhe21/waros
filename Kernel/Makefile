
# CONFIG: Architecture to build for
# ARCH ?= amd64
ARCH ?= x86
# ARCH ?= arm

DEBUG ?= yes

ifeq ($(ARCH),amd64)
    TRIPLE ?= x86_64-none-elf-
    GRUB_BUILD=yes
else ifeq ($(ARCH),x86)
    TRIPLE ?= i686-elf-
    GRUB_BUILD=yes
else ifeq ($(ARCH),arm)
    TRIPLE ?= arm-none-eabi-
    MACH ?= bcm
    # MACH ?= versatile
else
    $(error Unknown architecture $(ARCH))
endif


# Toolchain commands (can be overridden)
RUSTC ?= rustc
MKRESCUE ?= grub-mkrescue
LD := $(TRIPLE)ld
AS := $(TRIPLE)as
OBJDUMP := $(TRIPLE)objdump
OBJCOPY := $(TRIPLE)objcopy

# Object directory
OBJDIR := .obj/$(ARCH)/

LINKSCRIPT := arch/$(ARCH)/link.ld
TARGETSPEC := arch/$(ARCH)/target.json
# Compiler Options
LINKFLAGS := -T $(LINKSCRIPT)
LINKFLAGS += -Map $(OBJDIR)map.txt
LINKFLAGS += --gc-sections
LINKFLAGS += -z max-page-size=0x1000

RUSTFLAGS := --cfg arch__$(ARCH) --target=$(TARGETSPEC) --cfg disable_float
ifeq ($(DEBUG),yes)
    RUSTFLAGS += -g
else
    RUSTFLAGS += -O
endif

ifeq ($(ARCH),amd64)
    # - amd64 needs to be set to use soft floating point
    RUSTFLAGS += -C soft-float
else ifeq ($(ARCH),arm)
    RUSTFLAGS += --cfg target_mach=\"$(MACH)\"
    LINKFLAGS := --defsym MACH_$(MACH)=1 $(LINKFLAGS)
endif

# Objects
LIBCORE := $(OBJDIR)libcore.rlib
LIBALLOC_SYSTEM := $(OBJDIR)liballoc_system.rlib
LIBALLOC := $(OBJDIR)liballoc.rlib
LIBRUSTC_UNICODE := $(OBJDIR)librustc_unicode.rlib
LIBCOLLECTIONS := $(OBJDIR)libcollections.rlib
DEP_OBJS := libcore.rlib liballoc_system.rlib liballoc.rlib librustc_unicode.rlib libcollections.rlib
KERNEL_OBJS := start.o assembly.o kernel.o
OBJS := $(KERNEL_OBJS) $(DEP_OBJS)
OBJS := $(OBJS:%=$(OBJDIR)%)
BIN := ../kernel.$(ARCH).bin
DSM_BIN := $(BIN)
DSM := $(BIN).dsm
ifeq ($(GRUB_BUILD),yes)
    GRUB_DIR := ../grub/
    GRUB_CFG := $(GRUB_DIR)boot/grub/grub.cfg
    GRUB_IMG := $(GRUB_DIR)kernel.img
    GRUB_ISO := ../grub.$(ARCH).iso
endif
ifeq ($(ARCH),arm)
    DSM_BIN := $(BIN).elf
endif

.PHONY: all dsm clean distclean UPDATE

all: $(BIN) $(GRUB_ISO)

dsm: $(DSM)

clean:
	$(RM) -rf $(BIN) $(DSM) $(KERNEL_OBJS:%=$(OBJDIR)%) $(GRUB_ISO) $(GRUB_IMG)

distclean: clean
	$(RM) -rf $(OBJDIR)

UPDATE:
	curl https://static.rust-lang.org/dist/rustc-nightly-src.tar.gz -o ../rustc-nightly-src.tar.gz
NEWPATCH:
	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/libcore --transform 's~^rustc-nightly/src/libcore~libcore_orig~'
	cd .. && diff -rub libcore_orig/ libcore/ > libcore_nofp.patch || true

# Final link command
$(BIN): $(OBJS) $(LINKSCRIPT)
	$(LD) -o $@ $(LINKFLAGS) $(OBJS)
ifeq ($(ARCH),amd64)
	@mv $@ $@.elf64
	@$(OBJCOPY) $@.elf64 -F elf32-i386 $@
else ifeq ($(ARCH),arm)
	mv $@ $@.elf
	$(OBJCOPY) $@.elf -O binary $@
endif

$(GRUB_ISO): $(BIN) $(GRUB_CFG)
	cp $(BIN) $(GRUB_IMG)
	$(MKRESCUE) -o $@ $(GRUB_DIR)

$(DSM): $(BIN)
	$(OBJDUMP) -S $(DSM_BIN) > $@

../libcore/lib.rs: ../rustc-nightly-src.tar.gz ../libcore_nofp.patch
	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/libcore --transform 's~^rustc-nightly/src/~~'
	cd .. && patch -p0 < libcore_nofp.patch

../liballoc/lib.rs: ../rustc-nightly-src.tar.gz
	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/liballoc --transform 's~^rustc-nightly/src/~~'

../librustc_unicode/lib.rs: ../rustc-nightly-src.tar.gz
	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/librustc_unicode --transform 's~^rustc-nightly/src/~~'

../libcollections/lib.rs: ../rustc-nightly-src.tar.gz
	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/libcollections --transform 's~^rustc-nightly/src/~~'

# Compile libcore from ../libcore/
$(OBJDIR)libcore.rlib: ../libcore/lib.rs $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --out-dir $(OBJDIR) --crate-type=lib --emit=link,dep-info $<

# Compile liballoc from ../liballoc/
$(OBJDIR)liballoc.rlib: ../liballoc/lib.rs $(LIBCORE) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --out-dir $(OBJDIR) --crate-type=lib --emit=link,dep-info $< --extern core=$(LIBCORE)

# Compile librustc_unicode from ../librustc_unicode/
$(OBJDIR)librustc_unicode.rlib: ../librustc_unicode/lib.rs $(LIBCORE) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --out-dir $(OBJDIR) --crate-type=lib --emit=link,dep-info $< --extern core=$(LIBCORE)

# Compile libcollections from ../libcollections/
$(OBJDIR)libcollections.rlib: ../libcollections/lib.rs $(LIBCORE) $(LIBRUSTC_UNICODE) $(LIBALLOC) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --out-dir $(OBJDIR) --crate-type=lib --emit=link,dep-info $< --extern core=$(LIBCORE) --extern rustc_unicode=$(LIBRUSTC_UNICODE) --extern alloc=$(LIBALLOC)

# Compile a dummy crate for liballoc
$(OBJDIR)liballoc_system.rlib: ../AllocSystem/lib.rs $(LIBCORE) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --out-dir $(OBJDIR) --crate-type=lib --emit=link,dep-info $<

# Compile rust kernel object
$(OBJDIR)kernel.o: main.rs $(LIBCORE) $(LIBALLOC_SYSTEM) $(LIBALLOC) $(LIBRUSTC_UNICODE) $(LIBCOLLECTIONS) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --out-dir $(OBJDIR) --emit=obj,dep-info $< --extern core=$(LIBCORE) --extern alloc_system=$(LIBALLOC_SYSTEM) --extern alloc=$(LIBALLOC) --extern rustc_unicode=$(LIBRUSTC_UNICODE) --extern collections=$(LIBCOLLECTIONS)

# Compile architecture dependent startup routine
$(OBJDIR)start.o: arch/$(ARCH)/start.S
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<

include arch/$(ARCH)/assembly.mk

# Compile architecture's assembly stub
$(OBJDIR)assembly.o: ${ASMS:%=arch/$(ARCH)/%}
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $^

# Include dependency files
-include $(OBJDIR)libcore.d $(OBJDIR)liballoc.d $(OBJDIR)librustc_unicode.d $(OBJDIR)libcollections.d $(OBJDIR)liballoc_system.d $(OBJDIR)kernel.d
