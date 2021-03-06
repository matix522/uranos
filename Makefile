#
# MIT License
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

TARGET_RASPI3 = aarch64-unknown-none-raspi3
TARGET_RASPI4 = aarch64-unknown-none-raspi4


SOURCES = $(shell find src/ -type f -regex ".*\.rs") $(shell find src/ -type f -regex ".*\.S") link.ld Cargo.toml


XRUSTC_CMD_RASPI3   = cargo xbuild --target=.cargo/$(TARGET_RASPI3).json --release --features="raspi3"
CARGO_OUTPUT_RASPI3 = target/$(TARGET_RASPI3)/release/uranos


XRUSTC_CMD_RASPI4   = cargo xbuild --target=.cargo/$(TARGET_RASPI4).json --release --features="raspi4" 
CARGO_OUTPUT_RASPI4 = target/$(TARGET_RASPI4)/release/uranos

OBJCOPY        = rust-objcopy
OBJCOPY_PARAMS = --strip-all -O binary

CONTAINER_UTILS   = andrerichter/raspi3-utils

DOCKER_CMD        = docker run -it --rm --name uranos_qemu
DOCKER_CMD_DEBUG  = docker run -it --rm -p 1234:1234 --name uranos_qemu
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work

DOCKER_EXEC_QEMU     = qemu-system-aarch64 -M raspi3 -kernel bin/uranos-raspi3

.PHONY: all qemu clippy clean objdump nm

all:  bin/uranos-raspi4.img bin/uranos-raspi3.img


#### RASPBERRY PI3 ####
$(CARGO_OUTPUT_RASPI3): $(SOURCES)
	$(XRUSTC_CMD_RASPI3)

bin/uranos-raspi3.img: $(CARGO_OUTPUT_RASPI3)
	
	mkdir -p ./bin
	cp $< bin/uranos-raspi3
	$(OBJCOPY) $(OBJCOPY_PARAMS) bin/uranos-raspi3 bin/uranos-raspi3.img 

objdump-raspi3:
	rust-objdump --arch=aarch64 --disassemble --print-imm-hex bin/uranos-raspi3



#### RASPBERRY PI4 ####
$(CARGO_OUTPUT_RASPI4): $(SOURCES)
	$(XRUSTC_CMD_RASPI4)

bin/uranos-raspi4.img: $(CARGO_OUTPUT_RASPI4)
	mkdir -p ./bin
	cp $< ./bin/uranos-raspi4
	$(OBJCOPY) $(OBJCOPY_PARAMS) bin/uranos-raspi4 bin/uranos-raspi4.img

objdump-raspi4:
	rust-objdump --target .cargo/$(TARGET_RASPI4).json -- -disassemble -print-imm-hex bin/uranos-raspi4



#### QEMU WITH PI3 ####
qemu: all
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_QEMU) -serial stdio

qemu_debug: all
	$(DOCKER_CMD_DEBUG) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
		$(DOCKER_EXEC_QEMU) -serial stdio -gdb tcp:0.0.0.0:1234 -S

clippy:
	cargo xclippy --target=.cargo/$(TARGET_RASPI4).json

clean:
	cargo clean
	rm bin/*
nm:
	cargo nm --target ./.cargo/$(TARGET_RASPI4).json -- uranos-raspi4 | sort
