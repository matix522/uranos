ENTRY(_boot_cores);

SECTIONS
{
    . = 0x80000;

    .text :
    {
        KEEP(*(.text.boot)) *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
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

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}

PROVIDE(current_el0_synchronous   = default_interupt_handler);
PROVIDE(current_el0_irq           = default_interupt_handler);
PROVIDE(current_el0_serror        = default_interupt_handler);

PROVIDE(current_elx_synchronous   = default_interupt_handler);
PROVIDE(current_elx_irq           = default_interupt_handler);
PROVIDE(current_elx_serror        = default_interupt_handler);

PROVIDE(lower_aarch64_synchronous = default_interupt_handler);
PROVIDE(lower_aarch64_irq         = default_interupt_handler);
PROVIDE(lower_aarch64_serror      = default_interupt_handler);

PROVIDE(lower_aarch32_synchronous = default_interupt_handler);
PROVIDE(lower_aarch32_irq         = default_interupt_handler);
PROVIDE(lower_aarch32_serror      = default_interupt_handler);