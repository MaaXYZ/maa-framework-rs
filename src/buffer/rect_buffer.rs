use std::fmt::Debug;

use crate::internal;

pub struct MaaRectBuffer {
    pub(crate) handle: internal::MaaRectHandle,
    destroy_at_drop: bool,
}

impl Debug for MaaRectBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaaRectBuffer")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

impl From<internal::MaaRectHandle> for MaaRectBuffer {
    fn from(handle: internal::MaaRectHandle) -> Self {
        MaaRectBuffer {
            handle,
            destroy_at_drop: false,
        }
    }
}

unsafe impl Send for MaaRectBuffer {}
unsafe impl Sync for MaaRectBuffer {}

impl MaaRectBuffer {
    pub fn new() -> Self {
        let handle = unsafe { internal::MaaCreateRectBuffer() };

        MaaRectBuffer {
            handle,
            destroy_at_drop: true,
        }
    }

    pub fn x(&self) -> i32 {
        unsafe { internal::MaaGetRectX(self.handle) }
    }

    pub fn y(&self) -> i32 {
        unsafe { internal::MaaGetRectY(self.handle) }
    }

    pub fn width(&self) -> i32 {
        unsafe { internal::MaaGetRectW(self.handle) }
    }

    pub fn height(&self) -> i32 {
        unsafe { internal::MaaGetRectH(self.handle) }
    }

    pub fn set_x(self, x: i32) -> Self {
        unsafe {
            internal::MaaSetRectX(self.handle, x);
        };

        self
    }

    pub fn set_y(self, y: i32) -> Self {
        unsafe {
            internal::MaaSetRectY(self.handle, y);
        };

        self
    }

    pub fn set_width(self, width: i32) -> Self {
        unsafe {
            internal::MaaSetRectW(self.handle, width);
        }

        self
    }

    pub fn set_height(self, height: i32) -> Self {
        unsafe {
            internal::MaaSetRectH(self.handle, height);
        }

        self
    }
}

impl Default for MaaRectBuffer {
    fn default() -> Self {
        MaaRectBuffer::new()
    }
}

impl Drop for MaaRectBuffer {
    fn drop(&mut self) {
        if self.destroy_at_drop {
            unsafe {
                internal::MaaDestroyRectBuffer(self.handle);
            }
        }
    }
}
