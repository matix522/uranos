
TARGET = aarch64-unknown-none-softfloat

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) link.ld


XRUSTC_CMD   = cargo xrustc --target=$(TARGET) --release 
CARGO_OUTPUT = target/$(TARGET)/release/kernel8
CARGO_OUTPUT_RPI3 = target/$(TARGET)/release/kernel8-rpi3
CARGO_OUTPUT_RPI4 = target/$(TARGET)/release/kernel8-rpi4

OBJCOPY        = cargo objcopy --
OBJCOPY_PARAMS = --strip-all -O binary

CONTAINER_UTILS   = andrerichter/raspi3-utils

DOCKER_CMD        = docker run -it --rm
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work

DOCKER_EXEC_QEMU     = qemu-system-aarch64 -M raspi3 -kernel kernel8-rpi3.img

.PHONY: all qemu clippy clean objdump nm

all: clean kernel8-rpi4.img kernel8-rpi3.img

$(CARGO_OUTPUT_RPI3): $(SOURCES)
	$(XRUSTC_CMD) --features="raspi3"
	mv $(CARGO_OUTPUT) $(CARGO_OUTPUT_RPI3)

$(CARGO_OUTPUT_RPI4): $(SOURCES)
	$(XRUSTC_CMD)  --features="raspi4"
	mv $(CARGO_OUTPUT) $(CARGO_OUTPUT_RPI4)

kernel8-rpi4.img: $(CARGO_OUTPUT_RPI4)
	cp $< .
	$(OBJCOPY) $(OBJCOPY_PARAMS) $< kernel8-rpi4.img


kernel8-rpi3.img: $(CARGO_OUTPUT_RPI3)
	cp $< .
	$(OBJCOPY) $(OBJCOPY_PARAMS) $< kernel8-rpi3.img

qemu: all
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_QEMU) -d in_asm

clippy:
	cargo xclippy --target=$(TARGET)

clean:
	cargo clean

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex kernel8-rpi3

nm:
	cargo nm --target $(TARGET) -- kernel8-rpi3.img kernel8-rpi4.img | sort

