/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>,
 *               2019 Mateusz Hurbol <mateusz.hurbol42@gmail.com>, Piotr Kotara <piotrekkotara@gmail.com>
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

use core::ops;
use register::{mmio::*, register_bitfields};

// Auxilary mini UART registers
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Auxiliary enables
    AUX_ENABLES [
        /// If set the mini UART is enabled. The UART will immediately
        /// start receiving data, especially if the UART1_RX line is
        /// low.
        /// If clear the mini UART is disabled. That also disables any
        /// mini UART register access
        MINI_UART_ENABLE OFFSET(0) NUMBITS(1) []
    ],

    /// Mini Uart Interrupt Identify
    AUX_MU_IIR [
        /// Writing with bit 1 set will clear the receive FIFO
        /// Writing with bit 2 set will clear the transmit FIFO
        FIFO_CLEAR OFFSET(1) NUMBITS(2) [
            Rx = 0b01,
            Tx = 0b10,
            All = 0b11
        ]
    ],

    /// Mini Uart Line Control
    AUX_MU_LCR [
        /// Mode the UART works in
        DATA_SIZE OFFSET(0) NUMBITS(2) [
            SevenBit = 0b00,
            EightBit = 0b11
        ]
    ],

    /// Mini Uart Line Status
    AUX_MU_LSR [
        /// This bit is set if the transmit FIFO can accept at least
        /// one byte.
        TX_EMPTY   OFFSET(5) NUMBITS(1) [],

        /// This bit is set if the receive FIFO holds at least 1
        /// symbol.
        DATA_READY OFFSET(0) NUMBITS(1) []
    ],

    /// Mini Uart Extra Control
    AUX_MU_CNTL [
        /// If this bit is set the mini UART transmitter is enabled.
        /// If this bit is clear the mini UART transmitter is disabled.
        TX_EN OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// If this bit is set the mini UART receiver is enabled.
        /// If this bit is clear the mini UART receiver is disabled.
        RX_EN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Mini Uart Baudrate
    AUX_MU_BAUD [
        /// Mini UART baudrate counter
        RATE OFFSET(0) NUMBITS(16) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    __reserved_0: u32,                                  // 0x00
    AUX_ENABLES: ReadWrite<u32, AUX_ENABLES::Register>, // 0x04
    __reserved_1: [u32; 14],                            // 0x08
    AUX_MU_IO: ReadWrite<u32>,                          // 0x40 - Mini Uart I/O Data
    AUX_MU_IER: WriteOnly<u32>,                         // 0x44 - Mini Uart Interrupt Enable
    AUX_MU_IIR: WriteOnly<u32, AUX_MU_IIR::Register>,   // 0x48
    AUX_MU_LCR: WriteOnly<u32, AUX_MU_LCR::Register>,   // 0x4C
    AUX_MU_MCR: WriteOnly<u32>,                         // 0x50
    AUX_MU_LSR: ReadOnly<u32, AUX_MU_LSR::Register>,    // 0x54
    __reserved_2: [u32; 2],                             // 0x58
    AUX_MU_CNTL: WriteOnly<u32, AUX_MU_CNTL::Register>, // 0x60
    __reserved_3: u32,                                  // 0x64
    AUX_MU_BAUD: WriteOnly<u32, AUX_MU_BAUD::Register>, // 0x68
}

pub struct MiniUart {
    base_address: usize,
}

/// Deref to RegisterBlock
///
/// Allows writing
/// ```
/// self.MU_IER.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*MiniUart::ptr()).MU_IER.read() }
/// ```
impl ops::Deref for MiniUart {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl MiniUart {
    pub const fn new(base_address: usize) -> MiniUart {
        MiniUart { base_address }
    }

    /// Returns a pointer to the register block
    fn ptr(&self) -> *const RegisterBlock {
        self.base_address as *const _
    }
}
use crate::drivers::traits;
impl traits::Init  for MiniUart {
    ///Set baud rate and characteristics (115200 8N1) and map to GPIO
    fn init(&self) -> Result<(), &'static str> {
        // initialize UART
        self.AUX_ENABLES.modify(AUX_ENABLES::MINI_UART_ENABLE::SET);
        self.AUX_MU_IER.set(0);
        self.AUX_MU_CNTL.set(0);
        self.AUX_MU_LCR.write(AUX_MU_LCR::DATA_SIZE::EightBit);
        self.AUX_MU_MCR.set(0);
        self.AUX_MU_IER.set(0);
        self.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);
        self.AUX_MU_BAUD.write(AUX_MU_BAUD::RATE.val(270)); // 115200 baud

        use crate::drivers::gpio::*;
        use crate::utils::delay;

        let gpio = crate::drivers::GPIO.lock();
        gpio.GPFSEL1
            .modify(GPFSEL1::FSEL14::TXD1 + GPFSEL1::FSEL15::RXD1);
        gpio.GPPUD.set(0); // enable pins 14 and 15

        delay(1500);

        gpio.GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);

        delay(1500);

        gpio.GPPUDCLK0.set(0);

        self.AUX_MU_CNTL
            .write(AUX_MU_CNTL::RX_EN::Enabled + AUX_MU_CNTL::TX_EN::Enabled);
        Ok(())
    }
}
impl traits::console::Write for MiniUart {
    /// putc a character
    fn putb(&self, b: u8) {
        // wait until we can putc
        while !self.AUX_MU_LSR.is_set(AUX_MU_LSR::TX_EMPTY) {}

        // write the character to the buffer
        self.AUX_MU_IO.set(b as u32);
    }
}
impl traits::console::Read for MiniUart {
    /// Receive a byte character
    fn getb(&self) -> u8 {
        // wait until something is in the buffer
        while !self.AUX_MU_LSR.is_set(AUX_MU_LSR::DATA_READY) {}

        // read it
        let mut ret = self.AUX_MU_IO.get() as u8;

        // convert carrige return to newline
        if ret == b'\r' {
            ret = b'\n'
        }

        ret
    }
}
