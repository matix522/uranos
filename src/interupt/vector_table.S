.macro exception_handler handler, exception_level, id
.balign 0x80

    sub    sp,  sp,  #16 * 18

    stp    x0,  x1,  [sp, #16 * 0]
    stp    x2,  x3,  [sp, #16 * 1]
    stp    x4,  x5,  [sp, #16 * 2]
    stp    x6,  x7,  [sp, #16 * 3]
    stp    x8,  x9,  [sp, #16 * 4]
    stp    x10, x11, [sp, #16 * 5]
    stp    x12, x13, [sp, #16 * 6]
    stp    x14, x15, [sp, #16 * 7]
    stp    x16, x17, [sp, #16 * 8]
    stp    x18, x19, [sp, #16 * 9]
    stp    x20, x21, [sp, #16 * 10]
    stp    x22, x23, [sp, #16 * 11]
    stp    x24, x25, [sp, #16 * 12]
    stp    x26, x27, [sp, #16 * 13]
    stp    x28, x29, [sp, #16 * 14]

    mrs    x1,  SPSR_EL1
    mrs    x2,  ELR_EL1
    mrs    x3,  ESR_EL1
    .if \exception_level == 0
        mrs x4, SP_EL0 
    .else 
        add x4, sp, #16 * 18
    .endif

    stp    x30, x1,  [sp, #16 * 15]
    stp    x2,  x3,  [sp, #16 * 16]
    str    x4,       [sp, #16 * 17]

    mov    x0,  sp
    mov    x1, \id
    bl     \handler
    .if \exception_level == 0
        b restore_context_el0
    .else 
        b restore_context_el1
    .endif
.endm

.macro fiq_dummy
.balign 0x80
1:  wfe
    b      1b
.endm

.macro handle_invalid_entry type
	kernel_entry
	mov	x0, #\type
	mrs	x1, esr_el1
	mrs	x2, elr_el1
	bl	show_invalid_entry_message
	bl  blink
.endm


.global __irq_vector_init
__irq_vector_init:
    adr x0, __exception_vectors_start
    msr vbar_el1, x0
    ret

.globl __enable_irq
__enable_irq:
    msr    daifclr, #2 
    ret

.globl __disable_irq
__disable_irq:
    msr    daifset, #2
    ret

// The vector definitions
.section .vectors, "ax"
.global __exception_vectors_start
__exception_vectors_start:
    exception_handler current_el0_synchronous 1 0 // 0x000
    exception_handler current_el0_irq         1 1 // 0x080
    fiq_dummy                                                       // 0x100
    exception_handler current_el0_serror      1 2 // 0x180

    exception_handler current_elx_synchronous 1 3 // 0x200
    exception_handler current_elx_irq         1 4 // 0x280
    fiq_dummy                                                       // 0x300
    exception_handler current_elx_serror      1 5 // 0x380

    exception_handler lower_aarch64_synchronous 0 6 // 0x400
    exception_handler lower_aarch64_irq         0 7 // 0x480
    fiq_dummy                                                       // 0x500
    exception_handler lower_aarch64_serror      0 8 // 0x580

    exception_handler lower_aarch32_synchronous 0 9  // 0x600
    exception_handler lower_aarch32_irq         0 10 // 0x680
    fiq_dummy                                                       // 0x700
    exception_handler lower_aarch32_serror      0 11 // 0x780

.macro restore_context exception_level
    ldr    x18,      [sp, #16 * 17]
    ldp    x19, x21, [sp, #16 * 16]
    ldp    x30, x20, [sp, #16 * 15]

    msr    ESR_EL1,  x21
    msr    ELR_EL1,  x19
    msr    SPSR_EL1, x20

	.if	\exception_level == 0
	    msr	SP_EL0, x18
	.endif

    ldp    x0,  x1,  [sp, #16 * 0]
    ldp    x2,  x3,  [sp, #16 * 1]
    ldp    x4,  x5,  [sp, #16 * 2]
    ldp    x6,  x7,  [sp, #16 * 3]
    ldp    x8,  x9,  [sp, #16 * 4]
    ldp    x10, x11, [sp, #16 * 5]
    ldp    x12, x13, [sp, #16 * 6]
    ldp    x14, x15, [sp, #16 * 7]
    ldp    x16, x17, [sp, #16 * 8]
    ldp    x18, x19, [sp, #16 * 9]
    ldp    x20, x21, [sp, #16 * 10]
    ldp    x22, x23, [sp, #16 * 11]
    ldp    x24, x25, [sp, #16 * 12]
    ldp    x26, x27, [sp, #16 * 13]
    ldp    x28, x29, [sp, #16 * 14]

    add    sp,  sp,  #16 * 18

    eret
.endm
.global restore_context_el0
restore_context_el0:
    restore_context 0
.global restore_context_el1
restore_context_el1:
    restore_context 1


.global drop_el0
drop_el0:
    ldr    x18,      [x0, #16 * 17]
    ldp    x19, x21, [x0, #16 * 16]
    ldp    x30, x20, [x0, #16 * 15]

    msr    ESR_EL1,  x21
    msr    ELR_EL1,  x19
    msr    SPSR_EL1, x20
    msr	   SP_EL0,   x18

    ldp    x2,  x3,  [x0, #16 * 1]
    ldp    x4,  x5,  [x0, #16 * 2]
    ldp    x6,  x7,  [x0, #16 * 3]
    ldp    x8,  x9,  [x0, #16 * 4]
    ldp    x10, x11, [x0, #16 * 5]
    ldp    x12, x13, [x0, #16 * 6]
    ldp    x14, x15, [x0, #16 * 7]
    ldp    x16, x17, [x0, #16 * 8]
    ldp    x18, x19, [x0, #16 * 9]
    ldp    x20, x21, [x0, #16 * 10]
    ldp    x22, x23, [x0, #16 * 11]
    ldp    x24, x25, [x0, #16 * 12]
    ldp    x26, x27, [x0, #16 * 13]
    ldp    x28, x29, [x0, #16 * 14]
    ldp    x0,  x1,  [x0, #16 * 0]

    eret