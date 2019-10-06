pub enum Error{
    IrqNotEnabled
}
global_asm!(include_str!("vector_table.S"));