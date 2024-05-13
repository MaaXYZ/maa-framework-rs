use std::ffi::c_void;

use crate::{internal, maa_bool, Error, MaaResult};

pub struct MaaImageBuffer {
    pub(crate) handle: internal::MaaImageBufferHandle,
    destroy_at_drop: bool,
}

impl From<internal::MaaImageBufferHandle> for MaaImageBuffer {
    fn from(handle: internal::MaaImageBufferHandle) -> Self {
        MaaImageBuffer {
            handle,
            destroy_at_drop: false,
        }
    }
}

unsafe impl Send for MaaImageBuffer {}
unsafe impl Sync for MaaImageBuffer {}

impl MaaImageBuffer {
    pub fn new() -> Self {
        let handle = unsafe { internal::MaaCreateImageBuffer() };

        MaaImageBuffer {
            handle,
            destroy_at_drop: true,
        }
    }

    pub fn empty(&self) -> bool {
        let empty = unsafe { internal::MaaIsImageEmpty(self.handle) };

        maa_bool!(empty)
    }

    pub fn clear(&self) -> MaaResult<()> {
        let ret = unsafe { internal::MaaClearImage(self.handle) };
        maa_bool!(ret, BufferError)
    }

    pub fn get_raw(&self) -> *mut c_void {
        unsafe { internal::MaaGetImageRawData(self.handle) }
    }

    pub fn width(&self) -> i32 {
        unsafe { internal::MaaGetImageWidth(self.handle) }
    }

    pub fn height(&self) -> i32 {
        unsafe { internal::MaaGetImageHeight(self.handle) }
    }

    pub fn typ(&self) -> i32 {
        unsafe { internal::MaaGetImageType(self.handle) }
    }

    pub fn encoded(&self) -> *mut u8 {
        unsafe { internal::MaaGetImageEncoded(self.handle) }
    }

    pub fn encoded_size(&self) -> u64 {
        unsafe { internal::MaaGetImageEncodedSize(self.handle) }
    }

    /// # Safety
    ///
    /// data must be a valid pointer to a valid encoded image
    pub unsafe fn set_raw(&self, data: *mut c_void, width: i32, height: i32, typ: i32) {
        unsafe {
            internal::MaaSetImageRawData(self.handle, data, width, height, typ);
        }
    }

    /// # Safety
    ///
    /// data must be a valid pointer to a valid encoded image
    pub unsafe fn set_encoded(&self, data: *mut u8, size: u64) {
        unsafe {
            internal::MaaSetImageEncoded(self.handle, data, size);
        }
    }
}

impl Default for MaaImageBuffer {
    fn default() -> Self {
        MaaImageBuffer::new()
    }
}

impl Drop for MaaImageBuffer {
    fn drop(&mut self) {
        if self.destroy_at_drop {
            unsafe {
                internal::MaaDestroyImageBuffer(self.handle);
            }
        }
    }
}
