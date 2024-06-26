use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    fmt::Display,
    ops::Deref,
    ptr::null_mut,
};

use serde::{Deserialize, Serialize};
#[cfg(feature = "tokio")]
use tokio::task::JoinError;

pub use internal::MaaTaskId;

use crate::{
    CallbackHandler,
    controller::MaaControllerInstance,
    error,
    internal,
    maa_bool,
    MaaResult, MaaStatus, resource::MaaResourceInstance,
};
#[cfg(feature = "custom_action")]
use crate::custom::custom_action::{
    maa_custom_action_run, maa_custom_action_stop, MaaCustomAction,
};
#[cfg(feature = "custom_recognizer")]
use crate::custom::custom_recognizer::{custom_recognier_analyze, MaaCustomRecognizer};

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

pub struct MaaTaskResult<'a, T> {
    pub task_id: MaaTaskId,
    pub instance: &'a MaaInstance<T>,
}

impl<'a, T> MaaTaskResult<'a, T> {
    pub fn status(&self) -> MaaResult<MaaStatus> {
        self.instance.task_status(self.task_id)
    }

    pub fn wait(&self) -> MaaResult<MaaStatus> {
        self.instance.wait_task(self.task_id)
    }

    #[cfg(feature = "tokio")]
    #[doc(cfg(feature = "tokio"))]
    pub async fn wait_async(&self) -> Result<MaaResult<MaaStatus>, JoinError> {
        tokio::spawn(async move {
            self.instance.wait_task(self.task_id)
        }).await
    }

    pub fn set_task_param(&self, param: &str) -> MaaResult<()> {
        self.instance.set_task_param(self.task_id, param)
    }
}

/// The MaaInstance struct is the main entry point for the Maa library.
///
/// It is used to create and manage the Maa instance for running tasks.
///
/// # Example
///
/// ```
/// use maa_framework::instance::MaaInstance;
///
/// let instance = MaaInstance::new(None);
/// // let param = serde_json::json!({"param": "value"});
/// // instance.post_task("task", param).await;
/// ```
///
/// # Note
///
/// [MaaInstance], [MaaResourceInstance] and [MaaControllerInstance] use the same mechanism to manage the lifetime of the underlying C++ object.
/// That is, if the object is created from the Rust code (like `MaaInstance::new`), the object will be destroyed when it goes out of scope. In this case, it is your responsibility to ensure that the object is not used after it has been destroyed.
/// If the object is created from the C++ code, then you will not have to worry about the lifetime of the object.
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
    /// Create a new MaaInstance.
    ///
    /// # Parameters
    ///
    /// * `handler` - An optional callback handler for handling Maa events.
    pub fn new(handler: Option<T>) -> Self
    where
        T: CallbackHandler,
    {
        let handle = unsafe {
            match handler {
                Some(handler) => {
                    let callback_arg = Box::into_raw(Box::new(handler)) as *mut c_void;
                    internal::MaaCreate(Some(internal::callback_handler::<T>), callback_arg)
                }
                None => internal::MaaCreate(None, null_mut()),
            }
        };

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

    pub fn bind_resource(&self, res: &MaaResourceInstance<T>) -> MaaResult<()> {
        let ret = unsafe { internal::MaaBindResource(self.handle, res.handle) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceBindResourceError)
        }
    }

    pub fn bind_controller(&self, controller: &MaaControllerInstance<T>) -> MaaResult<()> {
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

    pub fn post_task<P>(&self, entry: &str, param: P) -> MaaTaskResult<T>
    where
        P: TaskParam,
    {
        let entry = CString::new(entry).unwrap();
        let param = param.get_param();
        let param = CString::new(param).unwrap();
        let task_id = unsafe { internal::MaaPostTask(self.handle, entry.as_ptr(), param.as_ptr()) };
        MaaTaskResult {
            task_id,
            instance: self,
        }
    }

    pub fn post_recognition(&self, entry: &str, param: &str) -> MaaTaskResult<T> {
        let entry = CString::new(entry).unwrap();
        let param = CString::new(param).unwrap();
        let task_id = unsafe { internal::MaaPostRecognition(self.handle, entry.as_ptr(), param.as_ptr()) };
        MaaTaskResult {
            task_id,
            instance: self,
        }
    }

    pub fn post_action(&self, entry: &str, param: &str) -> MaaTaskResult<T> {
        let entry = CString::new(entry).unwrap();
        let param = CString::new(param).unwrap();
        let task_id = unsafe { internal::MaaPostAction(self.handle, entry.as_ptr(), param.as_ptr()) };
        MaaTaskResult {
            task_id,
            instance: self,
        }
    }

    fn set_task_param(&self, task_id: MaaTaskId, param: &str) -> MaaResult<()> {
        let param = internal::to_cstring(param);
        let ret = unsafe { internal::MaaSetTaskParam(self.handle, task_id, param) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(error::Error::MaaInstanceSetTaskParamError(task_id))
        }
    }

    fn task_status(&self, task_id: MaaTaskId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaTaskStatus(self.handle, task_id) };

        MaaStatus::try_from(status)
    }

    fn wait_task(&self, task_id: MaaTaskId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaWaitTask(self.handle, task_id) };

        MaaStatus::try_from(status)
    }

    #[deprecated(note = "Use `running` instead")]
    pub fn task_all_finished(&self) -> bool {
        let ret = unsafe { internal::MaaTaskAllFinished(self.handle) };
        maa_bool!(ret)
    }

    pub fn running(&self) -> bool {
        let ret = unsafe { internal::MaaRunning(self.handle) };
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
    #[doc(cfg(feature = "custom_recognizer"))]
    pub fn register_custom_recognizer<R>(&mut self, name: &str, recognizer: R) -> MaaResult<()>
    where
        R: MaaCustomRecognizer,
    {
        let name_str = internal::to_cstring(name);
        let recognizer = Box::new(recognizer);
        let recognizer = Box::into_raw(recognizer) as *mut c_void;

        let recognizer_api = internal::MaaCustomRecognizerAPI {
            analyze: Some(custom_recognier_analyze::<R>),
        };

        let recognizer_api = Box::new(recognizer_api);
        let recognizer_api = Box::into_raw(recognizer_api) as *mut c_void;

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
    #[doc(cfg(feature = "custom_recognizer"))]
    pub fn unregister_custom_recognizer(&mut self, name: &str) -> MaaResult<()> {
        let name_str = internal::to_cstring(name);

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
    #[doc(cfg(feature = "custom_recognizer"))]
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
    #[doc(cfg(feature = "custom_action"))]
    pub fn register_custom_action<A>(&mut self, name: &str, action: A) -> MaaResult<()>
    where
        A: MaaCustomAction,
    {
        let name_str = internal::to_cstring(name);
        let action = Box::new(action);
        let action = Box::into_raw(action) as *mut c_void;

        let action_api = internal::MaaCustomActionAPI {
            run: Some(maa_custom_action_run::<A>),
            stop: Some(maa_custom_action_stop::<A>),
        };

        let action_api = Box::new(action_api);
        let action_api = Box::into_raw(action_api) as *mut c_void;

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
    #[doc(cfg(feature = "custom_action"))]
    pub fn unregister_custom_action(&mut self, name: &str) -> MaaResult<()> {
        let name_str = internal::to_cstring(name);

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
    #[doc(cfg(feature = "custom_action"))]
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
