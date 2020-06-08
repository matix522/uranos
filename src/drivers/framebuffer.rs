use crate::drivers::mbox;
use crate::utils::color::*;
use core::slice::from_raw_parts_mut as slice_form_raw;
use core::sync::atomic::{fence, Ordering};

pub struct FrameBuffer {
    pub height: usize,
    pub width: usize,
    pub pitch: usize,
    pub buffer: Option<&'static mut [u8]>,
}
pub enum FrameBufferError {
    MailboxError,
    UnsupportedResolution,
    ZeroSizedBuffer,
    UnsupportedDepth,
}
type FrameBufferResult = Result<(), &'static str>;

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> FrameBuffer {
        FrameBuffer {
            width,
            height,
            pitch: width * 4,
            buffer: None,
        }
    }
    pub fn init(&mut self) -> FrameBufferResult {
        let width = self.width as u32;
        let height = self.height as u32;

        let mut mbox_buffer = mbox::Mbox::make_buffer();

        mbox_buffer.buffer[0] = 35 * 4;
        mbox_buffer.buffer[1] = mbox::REQUEST;

        mbox_buffer.buffer[2] = mbox::tag::SET_PHYSICAL_SIZE; //set phy wh
        mbox_buffer.buffer[3] = 8;
        mbox_buffer.buffer[4] = 8;
        mbox_buffer.buffer[5] = width; //FrameBufferInfo.width
        mbox_buffer.buffer[6] = height; //FrameBufferInfo.height

        mbox_buffer.buffer[7] = mbox::tag::SET_VIRTUAL_SIZE; //set virt wh
        mbox_buffer.buffer[8] = 8;
        mbox_buffer.buffer[9] = 8;
        mbox_buffer.buffer[10] = width; //FrameBufferInfo.virtual_width
        mbox_buffer.buffer[11] = height; //FrameBufferInfo.virtual_height

        mbox_buffer.buffer[12] = mbox::tag::SET_VIRTUAL_OFFSET; //set virt offset
        mbox_buffer.buffer[13] = 8;
        mbox_buffer.buffer[14] = 8;
        mbox_buffer.buffer[15] = 0; //FrameBufferInfo.x_offset
        mbox_buffer.buffer[16] = 0; //FrameBufferInfo.y.offset

        mbox_buffer.buffer[17] = mbox::tag::SET_DEPTH; //set depth
        mbox_buffer.buffer[18] = 4;
        mbox_buffer.buffer[19] = 4;
        mbox_buffer.buffer[20] = 32; //FrameBufferInfo.depth

        mbox_buffer.buffer[21] = mbox::tag::SET_PIXEL_ORDER; //set pixel order
        mbox_buffer.buffer[22] = 4;
        mbox_buffer.buffer[23] = 4;
        mbox_buffer.buffer[24] = 1; //RGB, not BGR preferably

        mbox_buffer.buffer[25] = mbox::tag::ALLOCATE_FRAMEBUFFER; //get framebuffer, gets alignment on request
        mbox_buffer.buffer[26] = 8;
        mbox_buffer.buffer[27] = 8;
        mbox_buffer.buffer[28] = 4096; //FrameBufferInfo.pointer
        mbox_buffer.buffer[29] = 0; //FrameBufferInfo.size

        mbox_buffer.buffer[30] = mbox::tag::GET_PITCH; //get pitch
        mbox_buffer.buffer[31] = 4;
        mbox_buffer.buffer[32] = 4;
        mbox_buffer.buffer[33] = 0; //FrameBufferInfo.pitch

        mbox_buffer.buffer[34] = mbox::tag::LAST;

        // Insert a fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        fence(Ordering::Release);

        let response_buffer = {
            let mbox = crate::drivers::MBOX.lock();

            match mbox.call(mbox_buffer,mbox::channel::PROP) {
                Ok(response_buffer) =>  response_buffer,
                _  => return Err("Mailbox Error"),
            }
        };
    
        let height = response_buffer.buffer[6];
        let width = response_buffer.buffer[5];
        if width == 0 || height == 0 {
            return Err("FrameBufferError::UnsupportedResolution");
        }

        let buffer_address = response_buffer.buffer[28] & 0x3FFF_FFFF;
        let buffer_size = response_buffer.buffer[29];
        if buffer_address == 0 || buffer_size == 0 {
            return Err("FrameBufferError::ZeroSizedBuffer");
        }
        let depth = response_buffer.buffer[20];
        if depth != 32 {
            return Err("FrameBufferError::UnsupportedDepth");
        }
        let buffer = unsafe {
            slice_form_raw(buffer_address as usize as *mut u8, buffer_size as usize)
        };
        let pitch = response_buffer.buffer[33];

        self.width = width as usize;
        self.height = height as usize;
        self.pitch = pitch as usize;
        self.buffer = Some(buffer);
        Ok(())
    }
    pub fn set_pixel(&mut self, (x, y): (usize, usize), color: RGBA) {
        let pitch = self.pitch;
        let buffer = (&mut self.buffer)
            .as_mut()
            .expect("Buffer must be allocated");
        // crate::println!("({},{}) = ({},{},{},{})", x ,y ,r ,g, b, a );
        crate::aarch64::asm::nop();
        buffer[y * (pitch) + x * 4] = color.a;
        buffer[y * (pitch) + x * 4 + 1] = color.b;
        buffer[y * (pitch) + x * 4 + 2] = color.g;
        buffer[y * (pitch) + x * 4 + 3] = color.r;
        crate::aarch64::asm::nop();
    }
}
