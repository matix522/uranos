use core::ops::Range;
/// Translation types.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub enum Translation {
    Identity,
    Offset(usize),
}

/// Memory attributes.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub enum MemAttributes {
    CacheableDRAM,
    Device,
}

/// Access permissions.
#[derive(Copy, Clone, Debug)]
pub enum AccessPermissions {
    KernelReadOnly,
    KernelReadWrite,
    UserReadOnly,
    UserReadWrite,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Granule {
    Page4KiB,
    Block2MiB,
    Block1GiB,
}
/// Collection of memory attributes.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AccessPermissions,
    pub executable: bool,
}
impl AttributeFields {
    const fn new(
        mem_attributes: MemAttributes,
        acc_perms: AccessPermissions,
        executable: bool,
    ) -> Self {
        AttributeFields {
            mem_attributes,
            acc_perms,
            executable,
        }
    }
}
impl core::default::Default for AttributeFields {
    fn default() -> Self {
        AttributeFields::new(
            MemAttributes::CacheableDRAM,
            AccessPermissions::KernelReadWrite,
            false,
        )
    }
}

/// Static descriptor for a memory range.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct StaticRangeDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> Range<usize>,
    pub translation: Translation,
    pub attribute_fields: AttributeFields,
    pub granule: Granule,
}
impl StaticRangeDescriptor {
    pub const fn new(
        name: &'static str,
        virtual_range: fn() -> Range<usize>,
        translation: Translation,
        attribute_fields: AttributeFields,
        granule: Granule,
    ) -> Self {
        StaticRangeDescriptor {
            name,
            virtual_range,
            translation,
            attribute_fields,
            granule,
        }
    }
}

/// Descriptor for a memory range.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct RangeDescriptor {
    pub virtual_range: Range<usize>,
    pub translation: Translation,
    pub attribute_fields: AttributeFields,
    pub granule: Granule,
}
impl RangeDescriptor {
    pub const fn new(
        virtual_range: Range<usize>,
        translation: Translation,
        attribute_fields: AttributeFields,
        granule: Granule,
    ) -> Self {
        RangeDescriptor {
            virtual_range,
            translation,
            attribute_fields,
            granule,
        }
    }
}

pub const KERNEL_RW_: AttributeFields = AttributeFields::new(
    MemAttributes::CacheableDRAM,
    AccessPermissions::KernelReadWrite,
    false,
);
pub const KERNEL_R_X: AttributeFields = AttributeFields::new(
    MemAttributes::CacheableDRAM,
    AccessPermissions::KernelReadOnly,
    true,
);
pub const USER_RW_: AttributeFields = AttributeFields::new(
    MemAttributes::CacheableDRAM,
    AccessPermissions::UserReadWrite,
    false,
);
pub const USER_R_X: AttributeFields = AttributeFields::new(
    MemAttributes::CacheableDRAM,
    AccessPermissions::UserReadOnly,
    true,
);
pub const DEVICE: AttributeFields = AttributeFields::new(
    MemAttributes::Device,
    AccessPermissions::KernelReadWrite,
    false,
);

use crate::utils::binary_info::BinaryInfo;

pub const PHYSICAL_MEMORY_LAYOUT: [StaticRangeDescriptor; 7] = [
    StaticRangeDescriptor::new(
        "Init Stack",
        || {
            let binary_info = BinaryInfo::get();
            0..binary_info.binary.start
        },
        Translation::Identity,
        KERNEL_RW_,
        Granule::Page4KiB,
    ),
    StaticRangeDescriptor::new(
        "Static Kernel Data and Code",
        || {
            let binary_info = BinaryInfo::get();
            binary_info.read_only
        },
        Translation::Identity,
        USER_R_X,
        Granule::Page4KiB,
    ),
    StaticRangeDescriptor::new(
        "Mutable Init Task Data",
        || {
            let binary_info = BinaryInfo::get();
            binary_info.task_local
        },
        Translation::Identity,
        USER_RW_,
        Granule::Page4KiB,
    ),
    StaticRangeDescriptor::new(
        "Mutable Kernel Data",
        || {
            let binary_info = BinaryInfo::get();
            binary_info.read_write
        },
        Translation::Identity,
        USER_RW_,
        Granule::Page4KiB,
    ),
    StaticRangeDescriptor::new(
        "Allocator Page",
        || {
            let binary_info = BinaryInfo::get();
            binary_info.allocator
        },
        Translation::Identity,
        USER_RW_,
        Granule::Page4KiB,
    ),
    StaticRangeDescriptor::new(
        "Initial Kernel Heap",
        || {
            let binary_info = BinaryInfo::get();
            binary_info.heap
        },
        Translation::Identity,
        USER_RW_,
        Granule::Page4KiB,
    ),
    StaticRangeDescriptor::new(
        "MMIO devices",
        || {
            let binary_info = BinaryInfo::get();
            binary_info.mmio
        },
        Translation::Identity,
        DEVICE,
        Granule::Block2MiB,
    ),
];

use crate::sync::mutex::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;

pub type MemoryMap = BTreeMap<String, RangeDescriptor>;

pub static DYNAMIC_MEMORY_MAP_KERNEL: Mutex<MemoryMap> = Mutex::new(BTreeMap::new());
pub static DYNAMIC_MEMORY_MAP_USER: Mutex<MemoryMap> = Mutex::new(BTreeMap::new());

pub enum AddressSpace {
    Kernel,
    User,
}

use super::armv8::mmu::{map_memory, unmap_memory};

pub fn map_kernel_memory(
    memory_id: &str,
    virtual_range: Range<usize>,
    physical_start: usize,
    is_writable: bool,
) {
    let memory_range = RangeDescriptor::new(
        virtual_range,
        Translation::Offset(physical_start),
        if is_writable { KERNEL_RW_ } else { KERNEL_R_X },
        Granule::Page4KiB,
    );
    let mut map = DYNAMIC_MEMORY_MAP_KERNEL.lock();
    map.insert(memory_id.into(), memory_range)
        .expect_none("The name was already mapped to another memory");

    unsafe {
        map_memory(AddressSpace::Kernel, map.get(memory_id).unwrap()).unwrap();
    };
}

pub fn unmap_kernel_memory(memory_id: &str) {
    let mut map = DYNAMIC_MEMORY_MAP_KERNEL.lock();
    let mapped = map
        .remove(memory_id)
        .expect("The name does not match any kernel mapping.");
    unsafe {
        unmap_memory(AddressSpace::Kernel, &mapped).unwrap();
        llvm_asm!("tlbi vmalle1" : : : : "volatile");
    };
}

pub fn map_user_memory(
    memory_id: &str,
    virtual_range: Range<usize>,
    physical_start: usize,
    is_writable: bool,
) {
    let memory_range = RangeDescriptor::new(
        virtual_range,
        Translation::Offset(physical_start),
        if is_writable { USER_RW_ } else { USER_R_X },
        Granule::Page4KiB,
    );
    let mut map = DYNAMIC_MEMORY_MAP_USER.lock();
    map.insert(memory_id.into(), memory_range)
        .expect_none("The name was already mapped to another memory");

    unsafe {
        map_memory(AddressSpace::User, map.get(memory_id).unwrap()).unwrap();
    };
}

pub fn unmap_user_memory(memory_id: &str) {
    let mut map = DYNAMIC_MEMORY_MAP_USER.lock();
    let mapped = map
        .remove(memory_id)
        .expect("The name does not match any kernel mapping.");

    unsafe {
        unmap_memory(AddressSpace::User, &mapped).unwrap();
        llvm_asm!("tlbi vmalle1" : : : : "volatile");
    };
}
