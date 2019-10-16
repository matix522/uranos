use crate::mbox;
use alloc::boxed::Box;
use core::slice::from_raw_parts_mut as slice_form_raw;
use core::{
    ops,
    sync::atomic::{compiler_fence, Ordering},
};

pub struct FrameBuffer {
    buffer: Option<Box<[u8]>>,
    height: u32,
    width: u32,
}
pub enum FrameBufferError {
    MailboxError,
    UnsupportedResolution,
    ZeroSizedBuffer,
}
type FrameBufferResult = Result<(), FrameBufferError>;

impl FrameBuffer {
    pub const fn new() -> Self {
        FrameBuffer {
            buffer: None,
            height: 0,
            width: 0,
        }
    }
    pub fn init(&mut self, mbox: &mut mbox::Mbox) -> FrameBufferResult {
        mbox.buffer[0] = 13 * 4; // MSG SIZE
        mbox.buffer[1] = mbox::REQUEST; // REQUEST
        mbox.buffer[2] = mbox::tag::SET_PHYSICAL_SIZE; // TAG0: SET_PHYSICAL_SIZE
        mbox.buffer[3] = 8; // TAG0: VALUE SIZE
        mbox.buffer[4] = 0; // TAG0: CONTROL
        mbox.buffer[5] = 640; // TAG0 VALUE 0: PHYSICAL WIDTH
        mbox.buffer[6] = 480; // TAG0 VALUE 1: PHYSICAL HEIGHT
        mbox.buffer[7] = mbox::tag::ALLOCATE_FRAMEBUFFER; // TAG1: GET FRAMEBUFFER
        mbox.buffer[8] = 8; // TAG1: VALUE SIZE
        mbox.buffer[9] = 0; // TAG1: CONTROL
        mbox.buffer[10] = 0x8; // TAG1 VALUE 0: ALIGMENT / RESPONSE: ADDRESS
        mbox.buffer[11] = 0; // TAG1 VALUE 1: NONE / RESPONSE: SIZE
        mbox.buffer[12] = mbox::tag::LAST; // END MSG

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        match mbox.call(mbox::channel::PROP) {
            SUCCESS => {
                let height = mbox.buffer[5];
                let width = mbox.buffer[6];
                if width == 0 || height == 0 {
                    return Err(FrameBufferError::UnsupportedResolution);
                }
                let buffer_address = &mbox.buffer[10];
                let buffer_size = &mbox.buffer[11];
                if *buffer_address == 0 || *buffer_size == 0 {
                    return Err(FrameBufferError::ZeroSizedBuffer);
                }
                let buffer = unsafe {
                    slice_form_raw(*buffer_address as usize as *mut u8, *buffer_size as usize)
                };

                buffer[0] = 255;
                buffer[1] = 255;
                buffer[2] = 255;
                let boxed_buffer = unsafe { Box::from_raw(buffer) };
                self.buffer = Some(boxed_buffer);
                self.height = height;
                self.width = width;
                Ok(())
            }
            _ => Err(FrameBufferError::MailboxError),
        }
    }
}
