use std::{collections::HashMap, ffi::c_void, fmt::Display, ops::Deref, ptr::null_mut};

use serde::{Deserialize, Serialize};

use crate::{
    controller::MaaControllerInstance, error, internal, maa_bool, resource::MaaResourceInstance,
    string_view, CallbackHandler, MaaResult, MaaStatus,
};

#[cfg(feature = "custom_recognizer")]
use crate::custom::custom_recognizer::{custom_recognier_analyze, MaaCustomRecognizer};

#[cfg(feature = "custom_action")]
use crate::custom::custom_action::{
    maa_custom_action_run, maa_custom_action_stop, MaaCustomAction,
};

pub use internal::MaaTaskId;

pub trait TaskParam: Serialize {
    fn get_param(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl TaskParam for serde_json::Value {
    fn get_param(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaInstOption {
    Invalid,
}

impl Display for MaaInstOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaaInstOption::Invalid => write!(f, "Invalid"),
        }
    }
}

impl MaaInstOption {
    fn get_inner_key(&self) -> internal::MaaInstOption {
        match self {
            MaaInstOption::Invalid => internal::MaaInstOptionEnum_MaaInstOption_Invalid,
        }
    }
}

#[derive(Debug)]
pub struct MaaInstance<T> {
    pub(crate) handle: internal::MaaInstanceHandle,
    registered_custom_recognizers: HashMap<String, (*mut c_void, *mut c_void)>,
    registered_custom_actions: HashMap<String, (*mut c_void, *mut c_void)>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Deref for MaaInstance<T> {
    type Target = internal::MaaInstanceHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl<T> Send for MaaInstance<T> {}
unsafe impl<T> Sync for MaaInstance<T> {}

impl<T> MaaInstance<T> {
    pub fn new(handler: T) -> Self
    where
        T: CallbackHandler,
    {
        let callback_arg = Box::into_raw(Box::new(handler)) as *mut std::ffi::c_void;

        let handle =
            unsafe { internal::MaaCreate(Some(internal::callback_handler::<T>), callback_arg) };

        MaaInstance {
            handle,
            registered_custom_recognizers: HashMap::new(),
            registered_custom_actions: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set_option(&self, option: MaaInstOption) -> MaaResult<()> {
        let key = option.get_inner_key();

        let status = unsafe {
            match option {
                MaaInstOption::Invalid => internal::MaaSetOption(self.handle, key, null_mut(), 0),
            }
        };

        if status != 0 {
            Err(error::Error::MaaInstanceSetOptionError(option))
        } else {
            Ok(())
        }
    }

    pub fn bind_resource(&self, res: MaaResourceInstance<T>) -> MaaResult<()> {
        let ret = unsafe { internal::MaaBindResource(self.handle, res.handle) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceBindResourceError)
        }
    }

    pub fn bind_controller(&self, controller: MaaControllerInstance<T>) -> MaaResult<()> {
        let ret = unsafe { internal::MaaBindController(self.handle, controller.handle) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceBindControllerError)
        }
    }

    pub fn inited(&self) -> bool {
        let ret = unsafe { internal::MaaInited(self.handle) };
        maa_bool!(ret)
    }

    pub fn post_task<P>(&self, entry: &str, param: P) -> MaaTaskId
    where
        P: TaskParam,
    {
        let entry = string_view!(entry);
        let param = string_view!(param.get_param());
        unsafe { internal::MaaPostTask(self.handle, entry, param) }
    }

    pub fn set_task_param(&self, task_id: MaaTaskId, param: &str) -> MaaResult<()> {
        let param = string_view!(param);
        let ret = unsafe { internal::MaaSetTaskParam(self.handle, task_id, param) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceSetTaskParamError(task_id))
        }
    }

    pub fn task_status(&self, task_id: MaaTaskId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaTaskStatus(self.handle, task_id) };

        MaaStatus::try_from(status)
    }

    pub fn wait_task(&self, task_id: MaaTaskId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaWaitTask(self.handle, task_id) };

        MaaStatus::try_from(status)
    }

    pub fn task_all_finished(&self) -> bool {
        let ret = unsafe { internal::MaaTaskAllFinished(self.handle) };
        maa_bool!(ret)
    }

    pub fn post_stop(&self) -> MaaResult<()> {
        let ret = unsafe { internal::MaaPostStop(self.handle) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceStopError)
        }
    }

    pub fn stop(&self) -> MaaResult<()> {
        let ret = unsafe { internal::MaaStop(self.handle) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceStopError)
        }
    }

    pub fn resource(&self) -> MaaResourceInstance<T> {
        let handle = unsafe { internal::MaaGetResource(self.handle) };
        MaaResourceInstance::new_from_handle(handle)
    }

    pub fn controller(&self) -> MaaControllerInstance<T> {
        let handle = unsafe { internal::MaaGetController(self.handle) };
        MaaControllerInstance::new_from_handle(handle)
    }

    #[cfg(feature = "custom_recognizer")]
    pub fn register_custom_recognizer<R>(&mut self, name: &str, recognizer: R) -> MaaResult<()>
    where
        R: MaaCustomRecognizer,
    {
        let name_str = string_view!(name);
        let recognizer = Box::new(recognizer);
        let recognizer = Box::into_raw(recognizer) as *mut std::ffi::c_void;

        let recognizer_api = internal::MaaCustomRecognizerAPI {
            analyze: Some(custom_recognier_analyze::<R>),
        };

        let recognizer_api = Box::new(recognizer_api);
        let recognizer_api = Box::into_raw(recognizer_api) as *mut std::ffi::c_void;

        self.registered_custom_recognizers
            .insert(name.to_owned(), (recognizer, recognizer_api));

        let ret = unsafe {
            internal::MaaRegisterCustomRecognizer(
                self.handle,
                name_str,
                recognizer_api.cast(),
                recognizer,
            )
        };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceRegisterCustomRecognizerError(
                name.to_owned(),
            ))
        }
    }

    #[cfg(feature = "custom_recognizer")]
    pub fn unregister_custom_recognizer(&mut self, name: &str) -> MaaResult<()> {
        let name_str = string_view!(name);

        let (recognizer, recognizer_api) = self.registered_custom_recognizers.remove(name).unwrap();

        let ret = unsafe { internal::MaaUnregisterCustomRecognizer(self.handle, name_str) };

        unsafe {
            drop(Box::from_raw(
                recognizer as *mut Box<dyn MaaCustomRecognizer>,
            ));
            drop(Box::from_raw(
                recognizer_api as *mut internal::MaaCustomRecognizerAPI,
            ));
        }

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceUnregisterCustomRecognizerError(
                name.to_owned(),
            ))
        }
    }

    #[cfg(feature = "custom_recognizer")]
    pub fn clear_custom_recognizers(&mut self) -> MaaResult<()> {
        let ret = unsafe { internal::MaaClearCustomRecognizer(self.handle) };

        if !maa_bool!(ret) {
            return Err(error::Error::MaaInstanceClearCustomRecognizerError);
        }

        for (_name, (recognizer, recognizer_api)) in self.registered_custom_recognizers.drain() {
            unsafe {
                drop(Box::from_raw(
                    recognizer as *mut Box<dyn MaaCustomRecognizer>,
                ));
                drop(Box::from_raw(
                    recognizer_api as *mut internal::MaaCustomRecognizerAPI,
                ));
            }
        }

        Ok(())
    }

    #[cfg(feature = "custom_action")]
    pub fn register_custom_action<A>(&mut self, name: &str, action: A) -> MaaResult<()>
    where
        A: MaaCustomAction,
    {
        let name_str = string_view!(name);
        let action = Box::new(action);
        let action = Box::into_raw(action) as *mut std::ffi::c_void;

        let action_api = internal::MaaCustomActionAPI {
            run: Some(maa_custom_action_run::<A>),
            stop: Some(maa_custom_action_stop::<A>),
        };

        let action_api = Box::new(action_api);
        let action_api = Box::into_raw(action_api) as *mut std::ffi::c_void;

        self.registered_custom_actions
            .insert(name.to_owned(), (action, action_api));

        let ret = unsafe {
            internal::MaaRegisterCustomAction(self.handle, name_str, action_api.cast(), action)
        };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceRegisterCustomActionError(
                name.to_owned(),
            ))
        }
    }

    #[cfg(feature = "custom_action")]
    pub fn unregister_custom_action(&mut self, name: &str) -> MaaResult<()> {
        let name_str = string_view!(name);

        let (action, action_api) = self.registered_custom_actions.remove(name).unwrap();

        let ret = unsafe { internal::MaaUnregisterCustomAction(self.handle, name_str) };

        unsafe {
            drop(Box::from_raw(action as *mut Box<dyn MaaCustomAction>));
            drop(Box::from_raw(
                action_api as *mut internal::MaaCustomActionAPI,
            ));
        }

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceUnregisterCustomActionError(
                name.to_owned(),
            ))
        }
    }

    #[cfg(feature = "custom_action")]
    pub fn clear_custom_actions(&mut self) -> MaaResult<()> {
        let ret = unsafe { internal::MaaClearCustomAction(self.handle) };

        if !maa_bool!(ret) {
            return Err(error::Error::MaaInstanceClearCustomActionError);
        }

        for (_name, (action, action_api)) in self.registered_custom_actions.drain() {
            unsafe {
                drop(Box::from_raw(action as *mut Box<dyn MaaCustomAction>));
                drop(Box::from_raw(
                    action_api as *mut internal::MaaCustomActionAPI,
                ));
            }
        }

        Ok(())
    }
}

impl<T> Drop for MaaInstance<T> {
    fn drop(&mut self) {
        unsafe {
            internal::MaaDestroy(self.handle);
        }
    }
}
