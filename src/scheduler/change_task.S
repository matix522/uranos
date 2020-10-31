
.globl cpu_switch_to
cpu_switch_to:
    mov    x8, x0
    mov    x9, sp
    stp    x19, x20, [x8], #16        // store callee-saved registers
    stp    x21, x22, [x8], #16
    stp    x23, x24, [x8], #16
    stp    x25, x26, [x8], #16
    stp    x27, x28, [x8], #16
    stp    x29, x9, [x8], #16
    str    x30, [x8]

    mov    x8, x1
    ldp    x19, x20, [x8], #16        // restore callee-saved registers
    ldp    x21, x22, [x8], #16
    ldp    x23, x24, [x8], #16
    ldp    x25, x26, [x8], #16
    ldp    x27, x28, [x8], #16
    ldp    x29, x9, [x8], #16
    ldr    x30, [x8]
    mov    sp, x9
    ret
// 81E48
// 
.globl cpu_switch_to_first
cpu_switch_to_first:
    mov    x8, x0
    ldp    x19, x20, [x8], #16        // restore callee-saved registers
    ldp    x21, x22, [x8], #16
    ldp    x23, x24, [x8], #16
    ldp    x25, x26, [x8], #16
    ldp    x27, x28, [x8], #16
    ldp    x29, x9, [x8], #16
    ldr    x30, [x8]
    mov    sp, x9
    ret

.globl new_task_func
new_task_func:
    bl    schedule_tail
    mov    x0, x20
    mov    x1, x21
    mov    x2, xzr
    mov    x3, xzr
    mov    x4, xzr
    mov    x5, xzr
    mov    x6, xzr
    mov    x7, xzr
    mov    x8, xzr
    mov    x9, xzr
    mov    x10, xzr
    mov    x11, xzr
    mov    x12, xzr
    mov    x13, xzr
    mov    x14, xzr
    mov    x15, xzr
    mov    x16, xzr
    mov    x17, xzr
    mov    x18, xzr
    blr    x19         //should never return
    // bl terminate_task