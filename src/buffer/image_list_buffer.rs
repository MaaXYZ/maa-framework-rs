use crate::{internal, maa_bool, Error, MaaResult};

use super::image_buffer::MaaImageBuffer;

pub struct MaaImageListBuffer {
    pub(crate) handle: internal::MaaImageListBufferHandle,
    destroy_at_drop: bool,
}

impl From<internal::MaaImageListBufferHandle> for MaaImageListBuffer {
    fn from(handle: internal::MaaImageListBufferHandle) -> Self {
        Self {
            handle,
            destroy_at_drop: false,
        }
    }
}

unsafe impl Send for MaaImageListBuffer {}
unsafe impl Sync for MaaImageListBuffer {}

impl MaaImageListBuffer {
    pub fn new() -> Self {
        let handle = unsafe { internal::MaaCreateImageListBuffer() };

        Self {
            handle,
            destroy_at_drop: true,
        }
    }

    pub fn empty(&self) -> bool {
        let empty = unsafe { internal::MaaIsImageListEmpty(self.handle) };

        maa_bool!(empty)
    }

    pub fn clear(&self) -> MaaResult<()> {
        let ret = unsafe { internal::MaaClearImageList(self.handle) };

        maa_bool!(ret, BufferError)
    }

    pub fn size(&self) -> u64 {
        unsafe { internal::MaaGetImageListSize(self.handle) }
    }

    pub fn get(&self, index: u64) -> MaaImageBuffer {
        let handle = unsafe { internal::MaaGetImageListAt(self.handle, index) };

        MaaImageBuffer::from(handle)
    }

    pub fn append(&self, handle: MaaImageBuffer) -> MaaResult<()> {
        let ret = unsafe { internal::MaaImageListAppend(self.handle, handle.handle) };

        maa_bool!(ret, BufferError)
    }

    pub fn remove(&self, index: u64) -> MaaResult<()> {
        let ret = unsafe { internal::MaaImageListRemove(self.handle, index) };

        maa_bool!(ret, BufferError)
    }
}

impl Default for MaaImageListBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MaaImageListBuffer {
    fn drop(&mut self) {
        if self.destroy_at_drop {
            unsafe { internal::MaaDestroyImageListBuffer(self.handle) };
        }
    }
}
