
# CONFIG: Architecture to build for
# ARCH ?= amd64
ARCH ?= x86

DEBUG ?= yes

# Toolchain commands (can be overridden)
RUSTC ?= rustc
MKRESCUE ?= grub-mkrescue
ifeq ($(ARCH),amd64)
    LD := x86_64-elf-ld
    AS := x86_64-elf-as
    OBJDUMP := x86_64-elf-objdump
    OBJCOPY := x86_64-elf-objcopy
else ifeq ($(ARCH),x86)
    LD := i686-elf-ld
    AS := i686-elf-as
    OBJDUMP := i686-elf-objdump
else
    $(error Unknown architecture $(ARCH))
endif

# Object directory
OBJDIR := .obj/$(ARCH)/

LINKSCRIPT := arch/$(ARCH)/link.ld
TARGETSPEC := arch/$(ARCH)/target.json
# Compiler Options
LINKFLAGS := -T $(LINKSCRIPT)
LINKFLAGS += -Map $(OBJDIR)map.txt
LINKFLAGS += --gc-sections
LINKFLAGS += -z max-page-size=0x1000

RUSTFLAGS := --color always --cfg arch__$(ARCH) --target=$(TARGETSPEC)
ifeq ($(DEBUG),yes)
    RUSTFLAGS += -g -C opt-level=0
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
OBJS := start.o handlers.o kernel.o libcore.rlib liballoc.rlib
OBJS := $(OBJS:%=$(OBJDIR)%)
BIN := ../kernel.$(ARCH).bin
GRUB_DIR := ../grub/
GRUB_CFG := $(GRUB_DIR)boot/grub/grub.cfg
GRUB_IMG := $(GRUB_DIR)kernel.img
GRUB_ISO := ../grub.$(ARCH).iso

.PHONY: all clean

all: $(BIN) $(GRUB_ISO)

clean:
	$(RM) -rf $(BIN) $(BIN).dsm $(OBJDIR) $(GRUB_ISO) $(GRUB_IMG)

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

# Compile libcore from ../libcore/
$(OBJDIR)libcore.rlib: ../libcore/lib.rs $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --crate-type=lib --emit=link,dep-info $<

# Compile liballoc from ../liballoc/
$(OBJDIR)liballoc.rlib: ../liballoc/lib.rs $(LIBCORE) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) --cfg 'feature="external_funcs"' -o $@ --crate-type=lib --emit=link,dep-info $< --extern core=$(LIBCORE)

# Compile rust kernel object
$(OBJDIR)kernel.o: main.rs $(LIBCORE) $(LIBALLOC) $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --emit=obj,dep-info $< --extern core=$(LIBCORE) --extern alloc=$(LIBALLOC)

# Compile architecture's assembly stub
$(OBJDIR)start.o: arch/$(ARCH)/start.S
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<

# Compile architecture's assembly stub
$(OBJDIR)handlers.o: arch/$(ARCH)/interrupt/handlers.S
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<

# Include dependency files
-include $(OBJDIR)libcore.d $(OBJDIR)liballoc.d $(OBJDIR)kernel.d $(OBJDIR)start.d $(OBJDIR)handlers.d
