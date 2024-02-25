use std::{fmt::Display, ops::Deref, ptr::null_mut};

use crate::{
    buffer::string_buffer::MaaStringBuffer, internal, maa_bool, string_view, CallbackHandler,
    MaaResult, MaaStatus,
};

pub use internal::MaaResId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaResOption {
    Invalid,
}

impl MaaResOption {
    fn get_inner_key(&self) -> internal::MaaResOption {
        match self {
            MaaResOption::Invalid => internal::MaaResOptionEnum_MaaResOption_Invalid,
        }
    }
}

impl Display for MaaResOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaaResOption::Invalid => write!(f, "Invalid"),
        }
    }
}

#[derive(Debug)]
pub struct MaaResourceInstance<T> {
    pub(crate) handle: internal::MaaResourceHandle,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Deref for MaaResourceInstance<T> {
    type Target = internal::MaaResourceHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl<T> Send for MaaResourceInstance<T> {}
unsafe impl<T> Sync for MaaResourceInstance<T> {}

impl<T> MaaResourceInstance<T> {
    pub fn new(handler: Option<T>) -> Self
    where
        T: CallbackHandler,
    {
        let handle = unsafe {
            match handler {
                Some(handler) => {
                    let handler = Box::new(handler);
                    let handler = Box::into_raw(handler);
                    internal::MaaResourceCreate(
                        Some(internal::callback_handler::<T>),
                        handler.cast(),
                    )
                }
                None => internal::MaaResourceCreate(None, null_mut()),
            }
        };

        MaaResourceInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn new_from_handle(handle: internal::MaaResourceHandle) -> Self {
        MaaResourceInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn post_path(&self, path: &str) -> MaaResId {
        string_view!(path,path);
        unsafe { internal::MaaResourcePostPath(self.handle, path) }
    }

    pub fn status(&self, id: MaaResId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaResourceStatus(self.handle, id) };

        MaaStatus::try_from(status)
    }

    pub fn wait(&self, id: MaaResId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaResourceWait(self.handle, id) };

        MaaStatus::try_from(status)
    }

    pub fn loaded(&self) -> bool {
        let loaded = unsafe { internal::MaaResourceLoaded(self.handle) };
        maa_bool!(loaded)
    }

    pub fn set_option(&self, option: MaaResOption) -> MaaResult<()> {
        let key = option.get_inner_key();
        let ret = unsafe {
            match option {
                MaaResOption::Invalid => {
                    internal::MaaResourceSetOption(self.handle, key, null_mut(), 0)
                }
            }
        };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(crate::error::Error::MaaResourceSetOptionError(option))
        }
    }

    pub fn get_hash(&self) -> MaaResult<String> {
        let buffer = MaaStringBuffer::new();

        let ret = unsafe { internal::MaaResourceGetHash(self.handle, buffer.handle) };

        if maa_bool!(ret) {
            Ok(buffer.string())
        } else {
            Err(crate::error::Error::MaaResourceGetHashError)
        }
    }

    pub fn get_task_list(&self) -> MaaResult<String> {
        let buffer = MaaStringBuffer::new();

        let ret = unsafe { internal::MaaResourceGetTaskList(self.handle, buffer.handle) };

        if maa_bool!(ret) {
            let task_list = buffer.string();

            Ok(task_list)
        } else {
            Err(crate::error::Error::MaaResourceGetTaskListError)
        }
    }
}

impl<T> Drop for MaaResourceInstance<T> {
    fn drop(&mut self) {
        unsafe {
            internal::MaaResourceDestroy(self.handle);
        }
    }
}
