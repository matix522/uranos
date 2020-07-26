use cortex_a::regs::*;

use super::translation_tables::*;

struct MMU<const N: usize> {
    main_table: TopLevelTables<N>,
}
#[cfg(feature = "raspi3")]
unsafe fn get() -> alloc::boxed::Box<MMU<1>> {
    let m = alloc::boxed::Box::new_zeroed();
    m.assume_init()
}
#[cfg(not(feature = "raspi3"))]

unsafe fn get() -> alloc::boxed::Box<MMU<4>> {
    let m = alloc::boxed::Box::new_zeroed();
    m.assume_init()
}

#[repr(C, align(4096))]
struct TestTable<const N: usize> {
    table: [Table1Record; 512],
    blocks4G: [[PageRecord; 512]; N],
    blocks8G: [[PageRecord; 512]; N],


    table1G: [Table1Record; 512],     // EACH 1GB
    tables2M: [[Table1Record; 512]; N], // EACH 2MB
    tables2Ma: [[Table1Record; 512]; N], // MOVED VALUE 2MB
    pages4K: [[[PageRecord; 512];512]; N],    // EACH 4KB

    page: [PageRecord; N],
    // table: Table1Record,
    // blocks0: [PageRecord; 512],

}

impl<const N: usize> TestTable<N> {
    fn fill(&mut self) {
        let mut pages = &mut self.page;
        for (i, p) in pages.iter_mut().enumerate() {
            let addr = (1 << 30) * i;
            crate::println!("Range {:#018x}", addr);

            *p = if addr < 0x9000_0000 {
                PageRecord::new(addr, Default::default(), true)
            } else {
                use super::super::memory_controler::*;
                let a = AttributeFields {
                    acc_perms: AccessPermissions::ReadWrite,
                    mem_attributes: MemAttributes::Device,
                    execute_never: true,
                };
                PageRecord::new(addr, a, true)
            }
        }
        let mut blocks4G = &mut self.blocks4G;
        for (n, mut block1G) in blocks4G.iter_mut().enumerate() {
           
            for (i, p) in block1G.iter_mut().enumerate() {
                let addr = (1 << 30) * n + (1 << 21) * i;
                *p = if addr < 0x9000_0000 {
                    PageRecord::new(addr, Default::default(), true)
                } else {
                    use super::super::memory_controler::*;
                    let a = AttributeFields {
                        acc_perms: AccessPermissions::ReadWrite,
                        mem_attributes: MemAttributes::Device,
                        execute_never: true,
                    };
                    PageRecord::new(addr, a, true)
                }
            }
        }
        let mut blocks8G = &mut self.blocks8G;
        for (n, mut block1G) in blocks8G.iter_mut().enumerate() {
           
            for (i, p) in block1G.iter_mut().enumerate() {
                let addr = (1 << 30) * n + (1 << 21) * i;
                *p = if addr < 0x9000_0000 {
                    PageRecord::new(addr, Default::default(),true)
                } else {
                    use super::super::memory_controler::*;
                    let a = AttributeFields {
                        acc_perms: AccessPermissions::ReadWrite,
                        mem_attributes: MemAttributes::Device,
                        execute_never: true,
                    };
                    PageRecord::new(addr, a, true)
                }
            }
        }
        for (n, mut block1G) in self.pages4K.iter_mut().enumerate() {
            for (i, mut block2M) in block1G.iter_mut().enumerate() {
                for (j, mut page4K) in block2M.iter_mut().enumerate() {
                    let addr = (1 << 30) * n + (1 << 21) * i + (1 << 12) * j;
                    *page4K = if addr < crate::MMIO_BASE {
                        PageRecord::new(addr, Default::default(),false)
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
        for (n, t1G) in self.tables2M.iter_mut().enumerate() {
            for (i, t2M) in t1G.iter_mut().enumerate() {
                *t2M = self.pages4K[n][i].as_addr().into();
            }
        }
        for (n, t1G) in self.tables2Ma.iter_mut().enumerate() {
            for (i, t2M) in t1G.iter_mut().enumerate() {
                *t2M = self.pages4K[n][i].as_addr().into();
            }
        }
        for n in 0..N {
            self.table1G[n] = self.tables2M[n].as_addr().into(); 
        }
        for n in 0..N {
            self.table1G[n + N] = self.tables2Ma[n].as_addr().into(); 
        }
    }
}
#[cfg(feature = "raspi3")]
unsafe fn get_t() -> alloc::boxed::Box<TestTable<1>> {
    let m : alloc::boxed::Box<core::mem::MaybeUninit<TestTable<1>>> = alloc::boxed::Box::new_zeroed();
    let mut m = m.assume_init();
    m.fill();
    m
}

#[cfg(not(feature = "raspi3"))]
unsafe fn get_t() -> alloc::boxed::Box<TestTable<4>> {
    let m : alloc::boxed::Box<core::mem::MaybeUninit<TestTable<4>>> = alloc::boxed::Box::new_zeroed();
    let mut m = m.assume_init();
    m.fill();
    m
}

#[cfg(not(feature = "raspi3"))]
pub const MEMORY_SIZE : usize = 4;
#[cfg(feature = "raspi3")]
pub const MEMORY_SIZE : usize = 1;

fn translate<const N: usize>(virt_address: u64, m: &alloc::boxed::Box<MMU<N>>) -> u64 {
    let level_1_mask: u64 = 0b1_1111_1111 << 30;
    let level_2_mask: u64 = 0b1_1111_1111 << 21;
    let level_3_mask: u64 = 0b1_1111_1111 << 12;

    let level_1 = (virt_address & level_1_mask) as usize >> 30;
    let level_2 = (virt_address & level_2_mask) as usize >> 21;
    let level_3 = (virt_address & level_3_mask) as usize >> 12;
    unsafe {
        let address_2 =
            m.main_table.level_1[level_1].0 & (0b111_1111_1111_1111_1111_1111_1111 << 12);

        let address_3 =
            (*(address_2 as *const u64).add(level_2)) & (0b111_1111_1111_1111_1111_1111_1111 << 12);

        let address_p =
            (*(address_3 as *const u64).add(level_3)) & (0b111_1111_1111_1111_1111_1111_1111 << 12);
        address_p + (virt_address & 0b1111_1111_1111)
    }
}


fn test_translate<const N: usize>(virt_address: u64, m: &TestTable<N>) -> u64 {
    let level_1_mask: u64 = 0b1_1111_1111 << 30;
    let level_2_mask: u64 = 0b1_1111_1111 << 21;
    // let level_3_mask: u64 = 0b1_1111_1111 << 12;

    let level_1 = (virt_address & level_1_mask) as usize >> 30;
    let level_2 = (virt_address & level_2_mask) as usize >> 21;
    // let level_3 = (virt_address & level_3_mask) as usize >> 12;
    unsafe {
        let address_2 =
            m.table[level_1].0 & (0b111_1111_1111_1111_1111_1111_1111 << 12);

        let address_3 = *((address_2 as *const u64).add(level_2)) & (0b111_1111_1111_1111_1111_1111_1111 << 12);

        // let address_p =
        //     (*(address_3 as *const u64).add(level_3)) & (0b111_1111_1111_1111_1111_1111_1111 << 12);
        address_3 | (virt_address & 0b1_1111_1111_1111_1111_1111)
    }
}


pub unsafe fn test() -> Result<(), &'static str> {
    use cortex_a::barrier;
    let mut m = get();

    crate::println!("{:#018x}", &m.main_table as *const _ as u64);

    // Fail early if translation granule is not supported. Both RPis support it, though.
    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran4::Supported) {
        return Err("4 KiB translation granule not supported");
    }

    // Prepare the memory attribute indirection register.
    m.setup_mair();

    // Populate page tables.
    m.populate_tables();

    use alloc::boxed::Box;
    let a = 0x7F21_0000;
    // crate::println!("{:#018x}     {:#018x}", a, translate(a, &m));
    // crate::println!("L1 Table: {:#018x}", m.main_table.level_1.as_addr() as u64);

    let translation = get_t();



    let translation = Box::leak(translation);
    // page.page.0 += 1 << 1;
    // Set the "Translation Table Base Register".
    for (n, mut blocks) in translation.blocks4G.iter_mut().enumerate() {
        translation.table[n] = blocks.as_addr().into();
        crate::println!("Table {}",n);
    }
    let size = translation.blocks8G.len();
    for (n, mut blocks) in translation.blocks8G.iter_mut().enumerate() {
        translation.table[n + size] = blocks.as_addr().into();
        crate::println!("Table {}",n + size);
    }
    crate::println!("{:#018x}     {:#018x}", a, test_translate(a, &translation));


    // for t in translation.table.iter_mut() {
    //     crate::println!("table {:#018x}", t.0);

    // }    // translation.table0 = translation.blocks0.as_addr().into();

    let addr0 = translation.page.as_ptr() as u64;
    let addr1 = translation.table.as_ptr() as u64;
    let addr2 = translation.table1G.as_ptr() as u64;

    // for page in translation.page.iter() {
    //     crate::println!("{:#018x} --- {}", page.0, page);
    // }

    // let addr = m.main_table.level_1.as_addr() as u64;
    crate::println!("ADDR0: {:#018x}", addr0);
    crate::println!("ADDR1: {:#018x}", addr1);
    crate::println!("ADDR2: {:#018x}", addr2);

    TTBR0_EL1.set_baddr(addr2);
    TTBR1_EL1.set_baddr(addr2);

    m.configure_translation_control();
    Box::leak::<'static>(m);

    crate::println!("pre");

    // Switch the MMU on.
    //
    // First, force all previous changes to be seen before the MMU is enabled.
    barrier::isb(barrier::SY);

    // Enable the MMU and turn on data and instruction caching.
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    // Force MMU init to complete before next instruction.
    barrier::isb(barrier::SY);
    // writeln!(super::super::QEMUOutput {}, "QEMU TEST2");

    // SCTLR_EL1.modify(SCTLR_EL1::M::Disable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
    // // Force MMU init to complete before next instruction.
    // barrier::isb(barrier::SY);

    crate::println!("ggg");
    Ok(())
}
#[repr(C, align(4096))]
struct TopLevelTables<const N: usize> {
    level_3: [[[PageRecord; 512]; 512]; N], // Describing 4 KB each
    level_2: [[Table2Record; 512]; N],      // Describing 2 MB each
    level_1: [Table1Record; N],             // Describing 1024 MB each
}
impl<const N: usize> core::default::Default for TopLevelTables<N> {
    fn default() -> Self {
        TopLevelTables {
            level_1: [Table1Record(0); N],
            level_2: [[Table2Record(0); 512]; N],
            level_3: [[[PageRecord(0); 512]; 512]; N],
        }
    }
}

impl<const N: usize> MMU<N> {
    fn new() -> Self {
        MMU {
            main_table: Default::default(),
        }
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
    unsafe fn populate_tables(&mut self) {
        let tables = &mut self.main_table;
        crate::println!(
            "{} {} {}",
            tables.level_1.len(),
            tables.level_2.iter().fold(0, |x, y| x + y.len()),
            tables
                .level_3
                .iter()
                .fold(0, |x, y| x + y.iter().fold(0, |w, z| w + z.len()))
        );
        for (l1_nr, l1_entry) in tables.level_1.iter_mut().enumerate() {
            *l1_entry = tables.level_2[l1_nr].as_addr().into();
            // crate::println!("{:#018x}, {}", (l1_entry.0 >> ONE_GIB_SHIFT), l1_nr);
            for (l2_nr, l2_entry) in tables.level_2[l1_nr].iter_mut().enumerate() {
                *l2_entry = tables.level_3[l1_nr][l2_nr].as_addr().into();
                // crate::println!("{:#018x}, {:#018x}", (l1_entry.0 >> ONE_GIB_SHIFT), (l2_entry.0 >> TWO_MIB_SHIFT));
                // crate::println!("{:#018x}", (l1_nr << ONE_GIB_SHIFT) + (l2_nr << TWO_MIB_SHIFT));

                for (l3_nr, l3_entry) in tables.level_3[l1_nr][l2_nr].iter_mut().enumerate() {
                    let virt_addr = (l1_nr << ONE_GIB_SHIFT)
                        + (l2_nr << TWO_MIB_SHIFT)
                        + (l3_nr << FOUR_KIB_SHIFT);

                    let (output_addr, attribute_fields) = if virt_addr < 0x3F00_0000 {
                        (virt_addr, Default::default())
                    } else {
                        use crate::memory::memory_controler::*;
                        (
                            virt_addr,
                            AttributeFields {
                                mem_attributes: MemAttributes::Device,
                                acc_perms: AccessPermissions::ReadWrite,
                                execute_never: true,
                            },
                        )
                    };

                    *l3_entry = PageRecord::new(output_addr, attribute_fields, false);
                }
            }
        }
    }
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
