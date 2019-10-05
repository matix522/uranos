
use super::MMIO_BASE;
use crate::gpio;
use crate::mbox;
use core::{
    ops,
    sync::atomic::{compiler_fence, Ordering},
};
use register::{mmio::*, register_bitfields};
use core::sync::atomic::fence;

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

    /// Interupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

const UART_BASE: u32 = MMIO_BASE + 0x20_1000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: ReadWrite<u32>,                                                 // 0x00
    __reserved_0: [u32; 5],                                             // 0x04
    FLAG_REGISTER: ReadOnly<u32, FR::Register>,                         // 0x18
    __reserved_1: [u32; 2],                                             // 0x1c
    IBRD: WriteOnly<u32, IBRD::Register>,                               // 0x24
    FBRD: WriteOnly<u32, FBRD::Register>,                               // 0x28
    LINE_CONTROL_REGISTER: WriteOnly<u32, LCR::Register>,               // 0x2C
    CONTROL_REGISTER: WriteOnly<u32, CR::Register>,                     // 0x30
    __reserved_2: [u32; 4],                                             // 0x34
    INTERUPT_CLEAR_REGISTER: WriteOnly<u32, ICR::Register>,             // 0x44
}

pub enum UartError {
    MailboxError,
}
pub type UartResult = ::core::result::Result<(), UartError>;

pub struct Uart;

impl ops::Deref for Uart {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Uart {
    pub const fn new() -> Uart {
        Uart
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        UART_BASE as *const _
    }

    ///Set baud rate and characteristics (115200 8N1) and map to GPIO
    pub fn init(&self, mbox: &mut mbox::Mbox) -> UartResult {
        // turn off UART0
        self.CONTROL_REGISTER.set(0);

        // set up clock for consistent divisor values
        mbox.buffer[0] = 9 * 4;
        mbox.buffer[1] = mbox::REQUEST;
        mbox.buffer[2] = mbox::tag::SETCLKRATE;
        mbox.buffer[3] = 12;
        mbox.buffer[4] = 8;
        mbox.buffer[5] = mbox::clock::UART; // UART clock
        mbox.buffer[6] = 4_000_000; // 4Mhz
        mbox.buffer[7] = 0; // skip turbo setting
        mbox.buffer[8] = mbox::tag::LAST;

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        fence(Ordering::Release);

        if mbox.call(mbox::channel::PROP).is_err() {
            return Err(UartError::MailboxError); // Abort if UART clocks couldn't be set
        };

        // map UART0 to GPIO pins
        unsafe {
            (*gpio::GPFSEL1).modify(gpio::GPFSEL1::FSEL14::TXD0 + gpio::GPFSEL1::FSEL15::RXD0);

            (*gpio::GPPUD).set(0); // enable pins 14 and 15
            for _ in 0..150 {
                asm!("nop" :::: "volatile");
            }

            (*gpio::GPPUDCLK0).write(
                gpio::GPPUDCLK0::PUDCLK14::AssertClock + gpio::GPPUDCLK0::PUDCLK15::AssertClock,
            );
            for _ in 0..150 {
                asm!("nop" :::: "volatile");
            }

            (*gpio::GPPUDCLK0).set(0);
        }

        self.INTERUPT_CLEAR_REGISTER.write(ICR::ALL::CLEAR);
        self.IBRD.write(IBRD::IBRD.val(2)); // Results in 115200 baud
        self.FBRD.write(FBRD::FBRD.val(0xB));
        self.LINE_CONTROL_REGISTER.write(LCR::WLEN::EightBit); // 8N1
        self.CONTROL_REGISTER
            .write(CR::UART_ENABLE::Enabled + CR::TX_ENABLE::Enabled + CR::RX_ENABLE::Enabled);

        Ok(())
    }

    /// Send a character
    pub fn send(&self, c: char) {
        // wait until we can send
        loop {
            if !self.FLAG_REGISTER.is_set(FR::TX_FULL) {
                break;
            }

            unsafe { asm!("nop" :::: "volatile") };
        }

        // write the character to the buffer
        self.DR.set(c as u32);
    }

    /// Receive a character
    pub fn getc(&self) -> char {
        // wait until something is in the buffer
        loop {
            if !self.FLAG_REGISTER.is_set(FR::RX_EMPTY) {
                break;
            }

            unsafe { asm!("nop" :::: "volatile") };
        }

        // read it and return
        let mut ret = self.DR.get() as u8 as char;

        // convert carrige return to newline
        if ret == '\r' {
            ret = '\n'
        }

        ret
    }

    /// Display a string
    pub fn puts(&self, string: &str) {
        for c in string.chars() {
            // convert newline to carrige return + newline
            if c == '\n' {
                self.send('\r')
            }

            self.send(c);
        }
    }

    /// Display a binary value in hexadecimal
    pub fn hex(&self, d: u32) {
        let mut n;

        for i in 0..8 {
            // get highest tetrad
            n = d.wrapping_shr(28 - i * 4) & 0xF;

            // 0-9 => '0'-'9', 10-15 => 'A'-'F'
            // Add proper offset for ASCII table
            if n > 9 {
                n += 0x37;
            } else {
                n += 0x30;
            }

            self.send(n as u8 as char);
        }
    }
}
