use cortex_a::regs::*;

use super::translation_tables::*;

struct MMU {
    // main_table: TopLevelTables<N>,
}
unsafe fn get() -> alloc::boxed::Box<MMU> {
    let m = alloc::boxed::Box::new_zeroed();
    m.assume_init()
}

#[repr(C, align(4096))]
struct TestTable<const N: usize> {
    table_1g: [TableRecord; 512],            // EACH 1GB
    tables_2m: [[TableRecord; 512]; N],      // EACH 2MB
    tables_2m_a: [[TableRecord; 512]; N],    // MOVED VALUE 2MB
    pages_4k: [[[PageRecord; 512]; 512]; N], // EACH 4KB
}

impl<const N: usize> TestTable<N> {
    fn fill(&mut self) {
        for (n, block_1g) in self.pages_4k.iter_mut().enumerate() {
            for (i, block_2m) in block_1g.iter_mut().enumerate() {
                for (j, page_4k) in block_2m.iter_mut().enumerate() {
                    let addr = (1 << 30) * n + (1 << 21) * i + (1 << 12) * j;
                    *page_4k = if addr < crate::MMIO_BASE {
                        PageRecord::new(addr, Default::default(), false)
                    } else {
                        use super::super::memory_controler::*;
                        let a = AttributeFields {
                            acc_perms: AccessPermissions::ReadWrite,
                            mem_attributes: MemAttributes::Device,
                            execute_never: true,
                        };
                        PageRecord::new(addr, a, false)
                    }
                }
            }
        }
        for (n, table_1g) in self.tables_2m.iter_mut().enumerate() {
            for (i, table_2m) in table_1g.iter_mut().enumerate() {
                *table_2m = self.pages_4k[n][i].as_addr().into();
            }
        }
        for (n, table_1g) in self.tables_2m_a.iter_mut().enumerate() {
            for (i, table_2m) in table_1g.iter_mut().enumerate() {
                *table_2m = self.pages_4k[n][i].as_addr().into();
            }
        }
        for n in 0..N {
            self.table_1g[n] = self.tables_2m[n].as_addr().into();
        }
        for n in 0..N {
            self.table_1g[n + N] = self.tables_2m_a[n].as_addr().into();
        }
    }
}



#[cfg(not(feature = "raspi3"))]
pub const MEMORY_SIZE: usize = 4;
#[cfg(feature = "raspi3")]
pub const MEMORY_SIZE: usize = 1;

unsafe fn get_t() -> alloc::boxed::Box<TestTable<MEMORY_SIZE>> {
    let m: alloc::boxed::Box<core::mem::MaybeUninit<TestTable<MEMORY_SIZE>>> =
        alloc::boxed::Box::new_zeroed();
    let mut m = m.assume_init();
    m.fill();
    m
}
///
/// # Safety
/// Should be only called once before MMU is Initialized
pub unsafe fn test() -> Result<(), &'static str> {
    use cortex_a::barrier;
    let mut m = get();

    // Fail early if translation granule is not supported. Both RPis support it, though.
    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran4::Supported) {
        return Err("4 KiB translation granule not supported");
    }

    // Prepare the memory attribute indirection register.
    m.setup_mair();

    // Populate page tables.
    m.populate_tables();

    use alloc::boxed::Box;

    let translation = get_t();

    let translation = Box::leak(translation);

    let addr = translation.table_1g.as_ptr() as u64;

    crate::println!("MMU BASE TABLE: {:#018x}", addr);

    TTBR0_EL1.set_baddr(addr);
    TTBR1_EL1.set_baddr(addr);

    m.configure_translation_control();
    Box::leak::<'static>(m);

    crate::println!("Start MMU");

    // Switch the MMU on.
    //
    // First, force all previous changes to be seen before the MMU is enabled.
    barrier::isb(barrier::SY);

    // Enable the MMU and turn on data and instruction caching.
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    // Force MMU init to complete before next instruction.
    barrier::isb(barrier::SY);

    crate::println!("MMU Enabled");
    Ok(())
}

impl MMU {
    fn new() -> Self {
        MMU {}
    }

    unsafe fn setup_mair(&mut self) {
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
            MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +
            // Attribute 0 - Device. 
            MAIR_EL1::Attr0_Normal_Outer::Device +
            MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }
    unsafe fn populate_tables(&mut self) {}
    unsafe fn configure_translation_control(&mut self) {
        let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
        crate::println!("{}", ips);

        TCR_EL1.write(
            TCR_EL1::TBI0::Ignored
                + TCR_EL1::IPS.val(ips)

                + TCR_EL1::TG0.val(0b00)//::KiB_4
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::T0SZ.val(28) // TTBR0 spans 64 GiB total.

                + TCR_EL1::TG1.val(0b10)//::KiB_4
                + TCR_EL1::SH1::Inner
                + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD1::EnableTTBR1Walks
                + TCR_EL1::T1SZ.val(28), // TTBR1 spans 64 GiB total.
        );
    }
}

#[repr(u64)]
pub enum Mair {
    Device = 0,
    NormalCachableDRAM = 1,
}
