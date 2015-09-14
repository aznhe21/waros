
# CONFIG: Architecture to build for
# ARCH ?= amd64
ARCH ?= x86

DEBUG ?= yes

ifeq ($(ARCH),amd64)
    TRIPLE ?= x86_64-elf-
else ifeq ($(ARCH),x86)
    TRIPLE ?= i686-elf-
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

RUSTFLAGS := --color always --cfg arch__$(ARCH) --target=$(TARGETSPEC) --cfg disable_float
ifeq ($(DEBUG),yes)
    RUSTFLAGS += -g
else
    RUSTFLAGS += -O
endif
# - amd64 needs to be set to use soft floating point
ifeq ($(ARCH),amd64)
    RUSTFLAGS += -C soft-float
endif

# Objects
LIBCORE := $(OBJDIR)libcore.rlib
LIBALLOC := $(OBJDIR)liballoc.rlib
OBJS := start.o asm.o kernel.o libcore.rlib liballoc.rlib
OBJS := $(OBJS:%=$(OBJDIR)%)
BIN := ../kernel.$(ARCH).bin
GRUB_DIR := ../grub/
GRUB_CFG := $(GRUB_DIR)boot/grub/grub.cfg
GRUB_IMG := $(GRUB_DIR)kernel.img
GRUB_ISO := ../grub.$(ARCH).iso

.PHONY: all clean UPDATE

all: $(BIN) $(GRUB_ISO)

clean:
	$(RM) -rf $(BIN) $(BIN).dsm $(OBJDIR) $(GRUB_ISO) $(GRUB_IMG)

UPDATE:
	curl https://static.rust-lang.org/dist/rustc-nightly-src.tar.gz -o ../rustc-nightly-src.tar.gz

# Final link command
$(BIN): $(OBJS) arch/$(ARCH)/link.ld
	$(LD) -o $@ $(LINKFLAGS) $(OBJS)
	$(OBJDUMP) -S $@ > $@.dsm
ifeq ($(ARCH),amd64)
	@mv $@ $@.elf64
	@$(OBJCOPY) $@.elf64 -F elf32-i386 $@
endif

$(GRUB_ISO): $(BIN) $(GRUB_CFG)
	cp $(BIN) $(GRUB_IMG)
	$(MKRESCUE) -o $@ $(GRUB_DIR)

../libcore/lib.rs: ../rustc-nightly-src.tar.gz ../libcore_nofp.patch
	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/libcore --transform 's~^rustc-nightly/src/~~'
	cd .. && patch -p0 < libcore_nofp.patch

#../liballoc/lib.rs: ../rustc-nightly-src.tar.gz
#	tar -xmf ../rustc-nightly-src.tar.gz -C ../ rustc-nightly/src/liballoc --transform 's~^rustc-nightly/src/~~'

# Compile libcore from ../libcore/
$(OBJDIR)libcore.rlib: ../libcore/lib.rs $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --crate-type=lib --emit=link,dep-info $<

# Compile liballoc from ../liballoc/
$(OBJDIR)liballoc.rlib: ../liballoc/lib.rs $(LIBCORE) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --crate-type=lib --emit=link,dep-info $< --extern core=$(LIBCORE)

# Compile rust kernel object
$(OBJDIR)kernel.o: main.rs $(LIBCORE) $(LIBALLOC) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --emit=obj,dep-info $< --extern core=$(LIBCORE) --extern alloc=$(LIBALLOC)

# Compile architecture's assembly stub
$(OBJDIR)start.o: arch/$(ARCH)/start.S
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<

# Compile architecture's assembly stub
$(OBJDIR)asm.o: arch/$(ARCH)/memory.S arch/$(ARCH)/interrupt/handlers.S arch/$(ARCH)/interrupt/jump.S
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $^

# Include dependency files
-include $(OBJDIR)libcore.d $(OBJDIR)liballoc.d $(OBJDIR)kernel.d $(OBJDIR)start.d $(OBJDIR)asm.d
