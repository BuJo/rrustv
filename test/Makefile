PROJ = target

ARCH = rv32i
ABI = ilp32e
CCFLAGS = -Wall -fcommon -ffreestanding -O2
CCFLAGS += -march=$(ARCH) -mabi=$(ABI)
CCFLAGS += -D__RISCV__ -DBUILD="\"$(BUILD)\"" -DARCH="\"$(ARCH)\""
ASFLAGS = -march=$(ARCH)
LDFLAGS = -T$(PROJ).ld -Map=$(PROJ).map -m elf32lriscv -static -gc-sections --entry=_start # -Ttext=0
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

%.o: %.s Makefile
	$(AS) $(ASFLAGS) -c $< -o $@

%.ld: %.lds Makefile
	$(CPP) $(CPFLAGS) $< $@

.PHONY: clean all

$(PROJ).o: HelloWorld.o $(PROJ).ld
	$(LD) $(LDFLAGS) -o $@ HelloWorld.o

all: $(PROJ).text $(PROJ).data $(PROJ).uart
	$(OD) -s -d $(PROJ).o

clean:
	-rm -f HelloWorld.o $(PROJ).text $(PROJ).data $(PROJ).uart $(PROJ).ld

$(PROJ).text: $(PROJ).o
	$(OC) $(OCFLAGS) $< $(PROJ).text --only-section .*text* 
	@echo rom ok.
$(PROJ).data: $(PROJ).o
	$(OC) $(OCFLAGS) $< $(PROJ).data --only-section .*data* 
	@echo ram ok.
$(PROJ).uart: $(PROJ).o
	$(OC) $(OCFLAGS) $< $(PROJ).bin
	dd if=$(PROJ).bin of=$(PROJ).uart skip=512 bs=1
	rm $(PROJ).bin
	@echo mem ok.

