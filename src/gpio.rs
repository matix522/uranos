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

use super::MMIO_BASE;
use register::{mmio::ReadWrite, mmio::WriteOnly, register_bitfields};

register_bitfields! {
    u32,
    GPFSEL1 [
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
    GPPUDCLK0 [
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

pub const GPFSEL1: *const ReadWrite<u32, GPFSEL1::Register> =
    (MMIO_BASE + 0x0020_0004) as *const ReadWrite<u32, GPFSEL1::Register>;

pub const GPFSEL2: *const ReadWrite<u32, GPFSEL2::Register> =
    (MMIO_BASE + 0x0020_0008) as *const ReadWrite<u32, GPFSEL2::Register>;

pub const GPSET0: *const WriteOnly<u32, GPSET0::Register> =
    (MMIO_BASE + 0x0020_001C) as *const WriteOnly<u32, GPSET0::Register>;

pub const GPCLR0: *const WriteOnly<u32, GPCLR0::Register> =
    (MMIO_BASE + 0x0020_0028) as *const WriteOnly<u32, GPCLR0::Register>;

pub const GPPUD: *const ReadWrite<u32> = (MMIO_BASE + 0x0020_0094) as *const ReadWrite<u32>;

pub const GPPUDCLK0: *const ReadWrite<u32, GPPUDCLK0::Register> =
    (MMIO_BASE + 0x0020_0098) as *const ReadWrite<u32, GPPUDCLK0::Register>;

pub fn setup() {
    unsafe {
        (*GPFSEL2).modify(GPFSEL2::FSEL21::Output);

        (*GPPUD).set(0); // enable pin 21
        for _ in 0..150 {
            asm!("nop" :::: "volatile");
        }

        (*GPPUDCLK0).write(GPPUDCLK0::PUDCLK21::AssertClock);
        for _ in 0..150 {
            asm!("nop" :::: "volatile");
        }

        (*GPPUDCLK0).set(0);
    }
}

pub fn blink() -> ! {
    unsafe {
        loop {
            for _ in 0..150_000 {
                asm!("nop" :::: "volatile");
            }
            (*GPSET0).write(GPSET0::PIN21::Set);
            for _ in 0..150_000 {
                asm!("nop" :::: "volatile");
            }
            (*GPCLR0).write(GPCLR0::PIN21::Reset);
        }
    }
}
