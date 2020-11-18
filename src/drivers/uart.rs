/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 *               2019 Mateusz Hurbol <mateusz.hurbol42@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use super::mbox;
use core::{
    ops,
    sync::atomic::{fence, Ordering},
};
use register::{mmio::*, register_bitfields};

// PL011 UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Flag Register
    FR [
        /// Transmit FIFO full. The meaning of this bit depends on the
        /// state of the FEN bit in the UARTLCR_ LINE_CONTROL Register. If the
        /// FIFO is disabled, this bit is set when the transmit
        /// holding register is full. If the FIFO is enabled, the TXFULL
        /// bit is set when the transmit FIFO is full.
        TX_FULL OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the
        /// state of the FEN bit in the UARTLCR_H Register. If the
        /// FIFO is disabled, this bit is set when the receive holding
        /// register is empty. If the FIFO is enabled, the RX_EMPTY bit is
        /// set when the receive FIFO is empty.
        RX_EMPTY OFFSET(4) NUMBITS(1) []
    ],

    /// Integer Baud rate divisor
    IBRD [
        /// Integer Baud rate divisor
        IBRD OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud rate divisor
    FBRD [
        /// Fractional Baud rate divisor
        FBRD OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control register
    LCR [
        /// Word length. These bits indicate the number of data bits
        /// transmitted or received in a frame.
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register
    CR [
        /// Receive enable. If this bit is set to 1, the receive
        /// section of the UART is enabled.
        RX_ENABLE    OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        /// Transmit enable. If this bit is set to 1, the transmit
        /// section of the UART is enabled.
        TX_ENABLE    OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable
        UART_ENABLE OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission
            /// or reception, it completes the current character
            /// before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],
    /// Interrupt FIFO Level Select Register
    IFLS [
        /// Receive interrupt FIFO level select. The trigger points for the receive interrupt are as
        /// follows.
        RXIFLSEL OFFSET(3) NUMBITS(5) [
            OneEigth = 0b000,
            OneQuarter = 0b001,
            OneHalf = 0b010,
            ThreeQuarters = 0b011,
            SevenEights = 0b100
        ]
    ],

    /// Interrupt Mask Set Clear Register
    IMSC [
        /// Receive timeout interrupt mask. A read returns the current mask for the UARTRTINTR
        /// interrupt. On a write of 1, the mask of the interrupt is set. A write of 0 clears the
        /// mask.
        RTIM OFFSET(6) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Receive interrupt mask. A read returns the current mask for the UARTRXINTR interrupt. On
        /// a write of 1, the mask of the interrupt is set. A write of 0 clears the mask.
        RXIM OFFSET(4) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Masked Interrupt Status Register
    MIS [
        /// Receive timeout masked interrupt status. Returns the masked interrupt state of the
        /// UARTRTINTR interrupt.
        RTMIS OFFSET(6) NUMBITS(1) [],

        /// Receive masked interrupt status. Returns the masked interrupt state of the UARTRXINTR
        /// interrupt.
        RXMIS OFFSET(4) NUMBITS(1) []
    ],
    /// Interupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]

}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: ReadWrite<u32>,                                     // 0x00
    __reserved_0: [u32; 5],                                 // 0x04
    FLAG_REGISTER: ReadOnly<u32, FR::Register>,             // 0x18
    __reserved_1: [u32; 2],                                 // 0x1c
    IBRD: WriteOnly<u32, IBRD::Register>,                   // 0x24
    FBRD: WriteOnly<u32, FBRD::Register>,                   // 0x28
    LINE_CONTROL_REGISTER: WriteOnly<u32, LCR::Register>,   // 0x2C
    CONTROL_REGISTER: WriteOnly<u32, CR::Register>,         // 0x30
    IFLS: ReadWrite<u32, IFLS::Register>,                   // 0x34
    IMSC: ReadWrite<u32, IMSC::Register>,                   // 0x38
    __reserved_2: [u32; 1],                                 // 0x3C
    MIS: ReadOnly<u32, MIS::Register>,                      // 0x40
    INTERUPT_CLEAR_REGISTER: WriteOnly<u32, ICR::Register>, // 0x44
}

pub type UartResult = Result<(), &'static str>;

pub struct PL011Uart {
    pub base_address: usize,
}

impl ops::Deref for PL011Uart {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl PL011Uart {
    pub const fn new(base_address: usize) -> PL011Uart {
        PL011Uart { base_address }
    }

    /// Returns a pointer to the register block
    fn ptr(&self) -> *const RegisterBlock {
        self.base_address as *const _
    }

    pub fn move_uart(&mut self) {
        self.base_address |= crate::KERNEL_OFFSET + 0x1000;
    }
    pub fn get_base_address(&self) -> usize {
        self.base_address
    }

    pub fn register_and_enable_irq_handler(&'static self) -> Result<(), &'static str> {
        use super::rpi3_interrupt_controller::IRQType;
        use crate::interupts::interrupt_controller::InterruptController;

        let mut controler = crate::drivers::INTERRUPT_CONTROLLER.lock();

        let irq_descriptor = crate::interupts::IRQDescriptor {
            name: "PL011Uart",
            handler: Some(crate::interupts::handlers::uart_fn),
        };

        controler.connect_irq(IRQType::Uart, irq_descriptor)?;
        controler.enable_irq(IRQType::Uart)?;

        Ok(())
    }
}

use crate::drivers::traits;
impl traits::Init for PL011Uart {
    ///Set baud rate and characteristics (115200 8N1) and map to GPIO
    fn init(&self) -> UartResult {
        // turn off UART0
        self.CONTROL_REGISTER.set(0);
        let mut mbox_buffer = mbox::Mbox::make_buffer();
        // set up clock for consistent divisor values
        mbox_buffer.buffer[0] = 9 * 4;
        mbox_buffer.buffer[1] = mbox::REQUEST;
        mbox_buffer.buffer[2] = mbox::tag::SETCLKRATE;
        mbox_buffer.buffer[3] = 12;
        mbox_buffer.buffer[4] = 8;
        mbox_buffer.buffer[5] = mbox::clock::UART; // UART clock
        mbox_buffer.buffer[6] = 4_000_000; // 4Mhz
        mbox_buffer.buffer[7] = 0; // skip turbo setting
        mbox_buffer.buffer[8] = mbox::tag::LAST;

        // Insert a fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        fence(Ordering::Release);

        let _response_buffer = {
            let mbox = crate::drivers::MBOX.lock();

            match mbox.call(mbox_buffer, mbox::channel::PROP) {
                Ok(response_buffer) => response_buffer,
                _ => return Err("Mbox Error"),
            }
        };

        // map UART0 to GPIO pins
        use crate::drivers::gpio::*;
        use crate::utils::delay;

        let gpio = crate::drivers::GPIO.lock();
        gpio.GPFSEL1
            .modify(GPFSEL1::FSEL14::TXD0 + GPFSEL1::FSEL15::RXD0);
        gpio.GPPUD.set(0); // enable pins 14 and 15

        delay(1500);

        gpio.GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);

        delay(1500);

        gpio.GPPUDCLK0.set(0);

        // self.CONTROL_REGISTER.set(0);
        self.INTERUPT_CLEAR_REGISTER.write(ICR::ALL::CLEAR);
        self.IBRD.write(IBRD::IBRD.val(2)); // Results in 115200 baud
        self.FBRD.write(FBRD::FBRD.val(0xB));
        self.LINE_CONTROL_REGISTER.write(LCR::WLEN::EightBit); // + LCR::FEN::FifosEnabled); // 8N1
                                                               //self.IFLS.write(IFLS::RXIFLSEL::OneEigth); // RX FIFO fill level at 1/8
                                                               // self.IMSC.write(IMSC::RXIM::Enabled + IMSC::RTIM::Enabled); // RX IRQ + RX timeout IRQ

        self.CONTROL_REGISTER
            .write(CR::UART_ENABLE::Enabled + CR::TX_ENABLE::Enabled + CR::RX_ENABLE::Enabled);

        Ok(())
    }
}
impl traits::console::Write for PL011Uart {
    /// Send a character
    fn putb(&self, b: u8) {
        // wait until we can send
        while self.FLAG_REGISTER.is_set(FR::TX_FULL) {}

        // write the character to the buffer
        self.DR.set(b as u32);
    }
}
impl traits::console::Read for PL011Uart {
    /// Receive a byte character
    fn try_getb(&self) -> Option<u8> {
        // wait until something is in the buffer
        if self.FLAG_REGISTER.is_set(FR::RX_EMPTY) {
            return None;
        }

        // read byte
        let b = self.DR.get() as u8;

        // convert carrige return to newline
        if b == b'\r' {
            return Some(b'\n');
        }
        Some(b)
    }
}
