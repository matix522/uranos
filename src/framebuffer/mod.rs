pub mod charbuffer;

use crate::mbox;
use alloc::boxed::Box;
use core::slice::from_raw_parts_mut as slice_form_raw;
use core::{
    ops,
    sync::atomic::{compiler_fence, Ordering},
};

pub struct FrameBuffer {
    pub height: usize,
    pub width: usize,
    pub pitch: usize,
    pub buffer: &'static mut [u8],
}
pub enum FrameBufferError {
    MailboxError,
    UnsupportedResolution,
    ZeroSizedBuffer,
    UnsupportedDepth,
}
type FrameBufferResult = Result<FrameBuffer, FrameBufferError>;

impl FrameBuffer {
    // pub const fn new() -> Self {
    //     FrameBuffer {
    //         buffer: Box::new_uninit(),
    //         height: 0,
    //         width: 0,
    //         pitch: 0
    //     }
    // }
    pub fn new(mbox: &mut mbox::Mbox) -> FrameBufferResult {
        // mbox.buffer[0] = 13 * 4; // MSG SIZE
        // mbox.buffer[1] = mbox::REQUEST; // REQUEST
        // mbox.buffer[2] = mbox::tag::SET_PHYSICAL_SIZE; // TAG0: SET_PHYSICAL_SIZE
        // mbox.buffer[3] = 8; // TAG0: VALUE SIZE
        // mbox.buffer[4] = 0; // TAG0: CONTROL
        // mbox.buffer[5] = 640; // TAG0 VALUE 0: PHYSICAL WIDTH
        // mbox.buffer[6] = 480; // TAG0 VALUE 1: PHYSICAL HEIGHT
        // mbox.buffer[7] = mbox::tag::ALLOCATE_FRAMEBUFFER; // TAG1: GET FRAMEBUFFER
        // mbox.buffer[8] = 8; // TAG1: VALUE SIZE
        // mbox.buffer[9] = 0; // TAG1: CONTROL
        // mbox.buffer[10] = 0x8; // TAG1 VALUE 0: ALIGMENT / RESPONSE: ADDRESS
        // mbox.buffer[11] = 0; // TAG1 VALUE 1: NONE / RESPONSE: SIZE
        // mbox.buffer[12] = mbox::tag::LAST; // END MSG
        let width = 1024;
        let height = 768;

        mbox.buffer[0] = 35 * 4;
        mbox.buffer[1] = mbox::REQUEST;

        mbox.buffer[2] = mbox::tag::SET_PHYSICAL_SIZE; //set phy wh
        mbox.buffer[3] = 8;
        mbox.buffer[4] = 8;
        mbox.buffer[5] = width; //FrameBufferInfo.width
        mbox.buffer[6] = height; //FrameBufferInfo.height

        mbox.buffer[7] = mbox::tag::SET_VIRTUAL_SIZE; //set virt wh
        mbox.buffer[8] = 8;
        mbox.buffer[9] = 8;
        mbox.buffer[10] = width; //FrameBufferInfo.virtual_width
        mbox.buffer[11] = height; //FrameBufferInfo.virtual_height

        mbox.buffer[12] = mbox::tag::SET_VIRTUAL_OFFSET; //set virt offset
        mbox.buffer[13] = 8;
        mbox.buffer[14] = 8;
        mbox.buffer[15] = 0; //FrameBufferInfo.x_offset
        mbox.buffer[16] = 0; //FrameBufferInfo.y.offset

        mbox.buffer[17] = mbox::tag::SET_DEPTH; //set depth
        mbox.buffer[18] = 4;
        mbox.buffer[19] = 4;
        mbox.buffer[20] = 32; //FrameBufferInfo.depth

        mbox.buffer[21] = mbox::tag::SET_PIXEL_ORDER; //set pixel order
        mbox.buffer[22] = 4;
        mbox.buffer[23] = 4;
        mbox.buffer[24] = 1; //RGB, not BGR preferably

        mbox.buffer[25] = mbox::tag::ALLOCATE_FRAMEBUFFER; //get framebuffer, gets alignment on request
        mbox.buffer[26] = 8;
        mbox.buffer[27] = 8;
        mbox.buffer[28] = 4096; //FrameBufferInfo.pointer
        mbox.buffer[29] = 0; //FrameBufferInfo.size

        mbox.buffer[30] = mbox::tag::GET_PITCH; //get pitch
        mbox.buffer[31] = 4;
        mbox.buffer[32] = 4;
        mbox.buffer[33] = 0; //FrameBufferInfo.pitch

        mbox.buffer[34] = mbox::tag::LAST;

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        match mbox.call(mbox::channel::PROP) {
            SUCCESS => {
                let height = mbox.buffer[6];
                let width = mbox.buffer[5];
                if width == 0 || height == 0 {
                    return Err(FrameBufferError::UnsupportedResolution);
                }
                let buffer_address = mbox.buffer[28] & 0x3FFF_FFFF;
                let buffer_size = mbox.buffer[29];
                if buffer_address == 0 || buffer_size == 0 {
                    return Err(FrameBufferError::ZeroSizedBuffer);
                }
                let depth = mbox.buffer[20];
                if depth != 32 {
                    return Err(FrameBufferError::UnsupportedDepth);
                }
                let buffer = unsafe {
                    slice_form_raw(buffer_address as usize as *mut u8, buffer_size as usize)
                };
                let pitch = mbox.buffer[33] as usize;

                buffer[0] = 255;
                buffer[1] = 255;
                buffer[2] = 255;
                                // crate::println!("Test");

                // for y in 0..height as usize {
                //     for x in 0..(pitch) as usize {
                //         buffer[y * (pitch) + x] = ((y + x) % 256) as u8;  
                //         crate::println!("{}", x);          

                //     }
                //      crate::println!("{}", y);
                // }
                // crate::println!("DUPA");
                // unsafe { asm!("xD: nop" : : : : "volatile"); }
                // crate::println!("XD ");
                // // let boxed_buffer = unsafe { Box::from_raw(buffer) };
                // crate::println!("Test");
                Ok(FrameBuffer {
                    buffer: buffer,
                    height: height as usize,
                    width: width as usize,
                    pitch: mbox.buffer[33] as usize,
                })
            }
            _ => Err(FrameBufferError::MailboxError),
        }
    }
    pub fn set_pixel(&mut self, (x, y): (usize, usize), (r, g, b, a): (u8, u8, u8, u8)) {
        let pitch = self.pitch;
        let mut buffer = &mut self.buffer;
        // crate::println!("({},{}) = ({},{},{},{})", x ,y ,r ,g, b, a );
        unsafe {asm!("WRITE0: nop" : : : : "volatile")};
        buffer[y * (pitch) + x * 4] = a;
        buffer[y * (pitch) + x * 4 + 1] = b;
        buffer[y * (pitch) + x * 4 + 2] = g;
        buffer[y * (pitch) + x * 4 + 3] = r;
        unsafe {asm!("WRITE: nop" : : : : "volatile")};
    }
}