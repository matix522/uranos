.section ".text.boot"

.global _boot_cores

_boot_cores:
    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    // cpu id > 0, stop
1:  wfe
    b       1b
2:  // cpu id == 0

    // set stack before our code
    ldr     x1, =_boot_cores
    mov     sp, x1

    // jump to Rust code, should not return
    bl      reset
    // for failsafe, halt this core too
    b       1b