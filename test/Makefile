PROJ = target

ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
OUT = $(ROOT_DIR)/../target

ARCH = rv32i
ABI = ilp32e
CCFLAGS = -Wall -fcommon -ffreestanding -O2
CCFLAGS += -march=$(ARCH) -mabi=$(ABI)
CCFLAGS += -D__RISCV__ -DBUILD="\"$(BUILD)\"" -DARCH="\"$(ARCH)\""
ASFLAGS = -march=$(ARCH)
LDFLAGS = -T$(OUT)/$(PROJ).ld -Map=$(OUT)/$(PROJ).map -m elf32lriscv -static --entry=_start -gc-sections --print-gc-sections # -gc-sections -Ttext=0
CPFLAGS = -P
OCFLAGS = -O binary
ODFLAGS = -D

CCPATH = /usr/bin
CROSS = riscv64-linux-gnu
CC  = $(CCPATH)/$(CROSS)-gcc
AS  = $(CCPATH)/$(CROSS)-as
RL = $(CCPATH)/$(CROSS)-ranlib
LD  = $(CCPATH)/$(CROSS)-ld
OC  = $(CCPATH)/$(CROSS)-objcopy
OD  = $(CCPATH)/$(CROSS)-objdump
CPP = $(CCPATH)/$(CROSS)-cpp

all: $(OUT)/$(PROJ).text $(OUT)/$(PROJ).data $(OUT)/$(PROJ).uart
	$(OD) -s -d $(OUT)/$(PROJ).o

$(OUT)/%.o: %.s Makefile
	$(AS) $(ASFLAGS) -c $< -o $@

$(OUT)/%.ld: %.lds Makefile
	$(CPP) $(CPFLAGS) $< $@

.PHONY: clean all

$(OUT)/$(PROJ).o: $(OUT)/HelloWorld.o $(OUT)/$(PROJ).ld
	$(LD) $(LDFLAGS) -o $@ $<

clean:
	-rm -f $(OUT)/HelloWorld.o $(OUT)/$(PROJ).text $(OUT)/$(PROJ).data $(OUT)/$(PROJ).uart $(OUT)/$(PROJ).ld $(OUT)/$(PROJ).map

$(OUT)/$(PROJ).text: $(OUT)/$(PROJ).o
	$(OC) $(OCFLAGS) $< $(OUT)/$(PROJ).text --only-section .*text*
	@echo rom ok.
$(OUT)/$(PROJ).data: $(OUT)/$(PROJ).o
	$(OC) $(OCFLAGS) $< $(OUT)/$(PROJ).data --only-section .*data*
	@echo ram ok.
$(OUT)/$(PROJ).uart: $(OUT)/$(PROJ).o
	$(OC) $(OCFLAGS) $< $(OUT)/$(PROJ).bin
	dd if=$(OUT)/$(PROJ).bin of=$(OUT)/$(PROJ).uart skip=512 bs=1
	rm $(OUT)/$(PROJ).bin
	@echo mem ok.
