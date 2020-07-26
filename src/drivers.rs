pub mod charbuffer;
pub mod framebuffer;
pub mod gpio;
pub mod mbox;
pub mod miniuart;
pub mod uart;
pub mod rpi3_interrupt_controller;

pub mod traits;

macro_rules! device_driver {
    (synchronized $device_name : ident : $device_type : path = $initializer : expr) => {
        static_assertions::const_assert!(true);
    };
    (unsynchronized $device_name : ident : $device_type : path = $initializer : expr) => {
        device_driver_impl!($device_name: $device_type = $initializer);
    };
    ($device_name : ident : $device_type : path = $initializer : expr) => {
        device_driver_impl!($device_name: $device_type = $initializer);
    };
}

macro_rules! device_driver_impl {
    ($device_name : ident : $device_type : path = $initializer : expr) => {
        #[allow(non_snake_case)]
        pub(super) mod $device_name {
            use super::*;
            use crate::sync::nulllock::NullLock;
            #[link_section = ".devices"]
            static mut $device_name: Option<NullLock<$device_type>> = None;

            pub struct Get;

            impl core::ops::Deref for Get {
                type Target = NullLock<$device_type>;
                fn deref(&self) -> &Self::Target {
                    unsafe {
                        if ($device_name.is_some()) {
                            $device_name.as_ref().unwrap()
                        } else {
                            $device_name = Some(NullLock::new($initializer));
                            $device_name.as_ref().unwrap()
                        }
                    }
                }
            }
        }
        pub const $device_name: $device_name::Get = $device_name::Get {};
    };
}
device_driver!(
    unsynchronized MINIUART: miniuart::MiniUart = miniuart::MiniUart::new( crate::MMIO_BASE + 0x21_5000 )
);
device_driver!(
    unsynchronized MBOX: mbox::Mbox = mbox::Mbox::new(crate::MMIO_BASE + 0x00_B880)
);
device_driver!(
    unsynchronized UART: uart::PL011Uart = uart::PL011Uart::new(crate::MMIO_BASE + 0x20_1000)
);
device_driver!(
    unsynchronized GPIO: gpio::GpioType = gpio::GpioType::new(crate::MMIO_BASE + 0x20_0000)
);
device_driver!(
    unsynchronized FRAME_BUFFER: framebuffer::FrameBuffer = framebuffer::FrameBuffer::new(1024, 768)
);
device_driver!(
    unsynchronized CHAR_BUFFER: charbuffer::CharBuffer = charbuffer::CharBuffer::new()
);
