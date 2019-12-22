/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>,
 *               2019 Mateusz Hurbol <mateusz.hurbol42@gmail.com>, Piotr Kotara <piotrekkotara@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

ENTRY(_boot_cores);

SECTIONS
{
    . = 0x80000;

    __read_only_start = .;
    .text ALIGN(65536):
    {
        KEEP(*(.text.boot)) *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .vectors ALIGN(2048):
    {
        *(.vectors)
    }
    __read_only_end = .;

    .data ALIGN(65536):
    {
        *(.data .data.*)
    }

    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }
    .end ALIGN(8):
    {
        __binary_end = .;
    }

    .heap ALIGN(4096):
    {
        *(.heap .heap.*)
    }

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}

PROVIDE(current_el0_synchronous   = default_interupt_handler);
PROVIDE(current_el0_irq           = default_interupt_handler);
PROVIDE(current_el0_serror        = default_interupt_handler);

PROVIDE(current_elx_synchronous   = default_interupt_handler);
PROVIDE(current_elx_irq           = default_interupt_handler);
PROVIDE(current_elx_serror        = default_interupt_handler);

PROVIDE(lower_aarch64_synchronous = default_interupt_handler);
PROVIDE(lower_aarch64_irq         = current_elx_irq);
PROVIDE(lower_aarch64_serror      = default_interupt_handler);

PROVIDE(lower_aarch32_synchronous = default_interupt_handler);
PROVIDE(lower_aarch32_irq         = default_interupt_handler);
PROVIDE(lower_aarch32_serror      = default_interupt_handler);