use crate::{error::Error, internal, maa_bool, string, string_view, MaaResult};

pub struct MaaStringBuffer {
    pub(crate) handle: internal::MaaStringBufferHandle,
    destroy_at_drop: bool,
}

impl From<internal::MaaStringBufferHandle> for MaaStringBuffer {
    fn from(handle: internal::MaaStringBufferHandle) -> Self {
        MaaStringBuffer {
            handle,
            destroy_at_drop: false,
        }
    }
}

unsafe impl Send for MaaStringBuffer {}
unsafe impl Sync for MaaStringBuffer {}

impl MaaStringBuffer {
    pub fn new() -> Self {
        let handle = unsafe { internal::MaaCreateStringBuffer() };

        MaaStringBuffer {
            handle,
            destroy_at_drop: true,
        }
    }

    pub fn string(&self) -> String {
        let result = unsafe { internal::MaaGetString(self.handle) };

        string!(result)
    }

    pub fn set_string(&self, content: &str) -> MaaResult<()> {
        string_view!(content, content_str);
        let ret = unsafe { internal::MaaSetString(self.handle, content_str) };

        maa_bool!(ret, MaaSetStringError, content.to_owned())
    }
}

impl Default for MaaStringBuffer {
    fn default() -> Self {
        MaaStringBuffer::new()
    }
}

impl Drop for MaaStringBuffer {
    fn drop(&mut self) {
        if self.destroy_at_drop {
            unsafe {
                internal::MaaDestroyStringBuffer(self.handle);
            }
        }
    }
}
