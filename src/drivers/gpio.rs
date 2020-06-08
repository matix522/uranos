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
use register::{mmio::ReadWrite, mmio::WriteOnly, register_bitfields};
register_bitfields! {
    u32,
    pub GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            RXD0 = 0b100, // UART0     - Alternate function 0
            RXD1 = 0b010  // Mini UART - Alternate function 5

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            TXD0 = 0b100, // UART0     - Alternate function 0
            TXD1 = 0b010  // Mini UART - Alternate function 5
        ]
    ],
    /// GPIO Function Select 2
    GPFSEL2 [
        /// Pin 21
        FSEL21 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001
        ]
    ],
    /// GPIO Pull-up/down Clock Register 0
    pub GPPUDCLK0 [
        /// Pin 21
        PUDCLK21 OFFSET(21) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ],
    GPSET0 [
        PIN21 OFFSET(21) NUMBITS(1) [
            NoEffect = 0,
            Set = 1
        ]
    ],
    GPCLR0 [
        PIN21 OFFSET(21) NUMBITS(1) [
            NoEffect = 0,
            Reset = 1
        ]
    ]
}
pub struct GpioType {
    pub base_address: usize,
}

impl GpioType {
    pub fn new(base_address: usize) -> GpioType {
        GpioType { base_address }
    }
    fn ptr(&self) -> *const Registers {
        self.base_address as *const Registers
    }
}
use core::ops::Deref;
impl Deref for GpioType {
    type Target = Registers;
    fn deref(&self) -> &Registers {
        unsafe { &*self.ptr() }
    }
}
#[repr(C)]
#[allow(non_snake_case)]
pub struct Registers {
    _res0: [u8; 0x4],
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 0x04
    pub GPFSEL2: ReadWrite<u32, GPFSEL2::Register>, // 0x08
    _res1: [u8; 0x1C - 0x0C],                       // [0x0C - 0x1C)
    pub GPSET0: WriteOnly<u32, GPSET0::Register>,   // 0x1C
    _res2: [u8; 0x28 - 0x20],                       // [0x20 - 0x28)
    pub GPCLR0: WriteOnly<u32, GPCLR0::Register>,   // 0x28
    _res3: [u8; 0x94 - 0x2C],                       // [0x2C - 0x94)
    pub GPPUD: ReadWrite<u32>,                      // 0x94
    pub GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, // 0x98
}
