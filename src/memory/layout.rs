use super::AccessPermissions;
use super::AttributeFields;
use super::MemAttributes;
use core::fmt;
use core::ops::RangeInclusive;

#[derive(Copy, Clone)]
pub enum Translation {
    Identity,
    Offset(usize),
}

/// An architecture agnostic descriptor for a memory range.
pub struct RangeDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> RangeInclusive<usize>,
    pub translation: Translation,
    pub attribute_fields: AttributeFields,
}

/// Human-readable output of a RangeDescriptor.
impl fmt::Display for RangeDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Call the function to which self.range points, and dereference the result, which causes
        // Rust to copy the value.
        let start = *(self.virtual_range)().start();
        let end = *(self.virtual_range)().end();
        let size = end - start + 1;

        // log2(1024).
        const KIB_RSHIFT: u32 = 10;

        // log2(1024 * 1024).
        const MIB_RSHIFT: u32 = 20;

        let (size, unit) = if (size >> MIB_RSHIFT) > 0 {
            (size >> MIB_RSHIFT, "MiB")
        } else if (size >> KIB_RSHIFT) > 0 {
            (size >> KIB_RSHIFT, "KiB")
        } else {
            (size, "Byte")
        };

        let attr = match self.attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => "C",
            MemAttributes::Device => "Dev",
        };

        let acc_p = match self.attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => "RO",
            AccessPermissions::ReadWrite => "RW",
        };

        let xn = if self.attribute_fields.execute_never {
            "PXN"
        } else {
            "PX"
        };

        write!(
            f,
            "      {:#010x} - {:#010x} | {: >3} {} | {: <3} {} {: <3} | {}",
            start, end, size, unit, attr, acc_p, xn, self.name
        )
    }
}

/// Type for expressing the kernel's virtual memory layout.
pub struct KernelVirtualLayout<const NUM_SPECIAL_RANGES: usize> {
    max_virt_addr_inclusive: usize,
    inner: [RangeDescriptor; NUM_SPECIAL_RANGES],
}

impl<const NUM_SPECIAL_RANGES: usize> KernelVirtualLayout<{ NUM_SPECIAL_RANGES }> {
    pub const fn new(max: usize, layout: [RangeDescriptor; NUM_SPECIAL_RANGES]) -> Self {
        Self {
            max_virt_addr_inclusive: max,
            inner: layout,
        }
    }

    /// For a virtual address, find and return the output address and corresponding attributes.
    ///
    /// If the address is not found in `inner`, return an identity mapped default with normal
    /// cacheable DRAM attributes.
    pub fn get_virt_addr_properties(
        &self,
        virt_addr: usize,
    ) -> Result<(usize, AttributeFields), &'static str> {
        if virt_addr > self.max_virt_addr_inclusive {
            return Err("Address out of range");
        }

        for i in self.inner.iter() {
            if (i.virtual_range)().contains(&virt_addr) {
                let output_addr = match i.translation {
                    Translation::Identity => virt_addr,
                    Translation::Offset(a) => a + (virt_addr - (i.virtual_range)().start()),
                };

                return Ok((output_addr, i.attribute_fields));
            }
        }

        Ok((virt_addr, AttributeFields::default()))
    }

    /// Print the memory layout.
    pub fn print_layout(&self) {
        use crate::println;

        for i in self.inner.iter() {
            println!("{}", i);
        }
    }
}

const NUM_MEM_RANGES: usize = 2;
pub static LAYOUT: KernelVirtualLayout<{ NUM_MEM_RANGES }> = KernelVirtualLayout::new(
    super::physical::MEMORY_END,
    [
        RangeDescriptor {
            name: "Kernel code and readonly data",
            virtual_range: || unsafe {
                let read_only_start = &crate::__read_only_start as *const usize as usize;
                let read_only_end = &crate::__read_only_end as *const usize as usize - 1;

                // crate::println!("Read only at: {:x} - {:x}", read_only_start, read_only_end);
                RangeInclusive::new(read_only_start, read_only_end)
            },
            translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadOnly,
                execute_never: false,
            },
        },
        // RangeDescriptor {
        //     name: "Interupt VectorTable",
        //     virtual_range: || unsafe {
        //         let read_only_start = &crate::__read_only_start as *const usize as usize;
        //         let read_only_end = &crate::__read_only_end as *const usize as usize - 1;

        //         // crate::println!("Read only at: {:x} - {:x}", read_only_start, read_only_end);
        //         RangeInclusive::new(read_only_start, read_only_end)
        //     },
        //     translation: Translation::Identity,
        //     attribute_fields: AttributeFields {
        //         mem_attributes: MemAttributes::CacheableDRAM,
        //         acc_perms: AccessPermissions::ReadOnly,
        //         execute_never: false,
        //     },
        // },
        // __exception_vectors_start
        RangeDescriptor {
            name: "Device MMIO",
            virtual_range: || {
                RangeInclusive::new(super::physical::mmio::BASE, super::physical::mmio::END)
            },
            translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
    ],
);
