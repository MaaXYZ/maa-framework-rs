//! Custom recognizer and action components.
//!
//! This module provides traits for implementing custom recognition algorithms
//! and actions that integrate with the MaaFramework pipeline system.
//!
//! # Usage
//!
//! 1. Implement [`CustomRecognition`] or [`CustomAction`] trait
//! 2. Register with [`Resource::register_custom_recognition`] or [`Resource::register_custom_action`]
//! 3. Reference in pipeline JSON via `"recognition": "Custom"` or `"action": "Custom"`

use crate::context::Context;
use crate::resource::Resource;
use crate::{common, sys, MaaError, MaaResult};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

// === Trait Definitions ===

/// Trait for implementing custom actions.
///
/// Actions are executed after a successful recognition and can perform
/// clicks, swipes, or custom logic.
pub trait CustomAction: Send + Sync {
    /// Execute the custom action.
    ///
    /// # Arguments
    /// * `context` - The current execution context
    /// * `task_id` - ID of the current task
    /// * `node_name` - Name of the current node
    /// * `custom_action_name` - Name of this action (as registered)
    /// * `custom_action_param` - JSON parameters for this action
    /// * `reco_id` - ID of the preceding recognition result
    /// * `box_rect` - Target region found by recognition
    ///
    /// # Returns
    /// `true` if the action succeeded, `false` otherwise
    fn run(
        &self,
        context: &Context,
        task_id: common::MaaId,
        node_name: &str,
        custom_action_name: &str,
        custom_action_param: &str,
        reco_id: common::MaaId,
        box_rect: &common::Rect,
    ) -> bool;
}

/// Trait for implementing custom recognizers.
///
/// Recognizers analyze screenshots to find targets in the UI.
pub trait CustomRecognition: Send + Sync {
    /// Analyze an image to find the target.
    ///
    /// # Arguments
    /// * `context` - The current execution context
    /// * `task_id` - ID of the current task
    /// * `node_name` - Name of the current node
    /// * `custom_recognition_name` - Name of this recognizer
    /// * `custom_recognition_param` - JSON parameters for this recognizer
    /// * `image` - The image to analyze
    /// * `roi` - Region of Interest to restrict analysis
    ///
    /// # Returns
    /// `Some((rect, detail))` if target found, `None` otherwise
    fn analyze(
        &self,
        context: &Context,
        task_id: common::MaaId,
        node_name: &str,
        custom_recognition_name: &str,
        custom_recognition_param: &str,
        image: &crate::buffer::MaaImageBuffer,
        roi: &common::Rect,
    ) -> Option<(common::Rect, String)>;
}

// === Function Wrapper for Recognition ===

/// Wrapper to use a closure as a custom recognition.
///
/// This allows using a simple closure instead of implementing a full trait.
pub struct FnRecognition<F>
where
    F: Fn(&Context, &RecognitionArgs) -> Option<(common::Rect, String)> + Send + Sync,
{
    func: F,
}

/// Arguments bundle for custom recognition closure.
pub struct RecognitionArgs<'a> {
    pub task_id: common::MaaId,
    pub node_name: &'a str,
    pub name: &'a str,
    pub param: &'a str,
    pub image: &'a crate::buffer::MaaImageBuffer,
    pub roi: &'a common::Rect,
}

impl<F> FnRecognition<F>
where
    F: Fn(&Context, &RecognitionArgs) -> Option<(common::Rect, String)> + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> CustomRecognition for FnRecognition<F>
where
    F: Fn(&Context, &RecognitionArgs) -> Option<(common::Rect, String)> + Send + Sync,
{
    fn analyze(
        &self,
        context: &Context,
        task_id: common::MaaId,
        node_name: &str,
        custom_recognition_name: &str,
        custom_recognition_param: &str,
        image: &crate::buffer::MaaImageBuffer,
        roi: &common::Rect,
    ) -> Option<(common::Rect, String)> {
        let args = RecognitionArgs {
            task_id,
            node_name,
            name: custom_recognition_name,
            param: custom_recognition_param,
            image,
            roi,
        };
        (self.func)(context, &args)
    }
}

// === Function Wrapper for Action ===

/// Wrapper to use a closure as a custom action.
pub struct FnAction<F>
where
    F: Fn(&Context, &ActionArgs) -> bool + Send + Sync,
{
    func: F,
}

/// Arguments bundle for custom action closure.
pub struct ActionArgs<'a> {
    pub task_id: common::MaaId,
    pub node_name: &'a str,
    pub name: &'a str,
    pub param: &'a str,
    pub reco_id: common::MaaId,
    pub box_rect: &'a common::Rect,
}

impl<F> FnAction<F>
where
    F: Fn(&Context, &ActionArgs) -> bool + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> CustomAction for FnAction<F>
where
    F: Fn(&Context, &ActionArgs) -> bool + Send + Sync,
{
    fn run(
        &self,
        context: &Context,
        task_id: common::MaaId,
        node_name: &str,
        custom_action_name: &str,
        custom_action_param: &str,
        reco_id: common::MaaId,
        box_rect: &common::Rect,
    ) -> bool {
        let args = ActionArgs {
            task_id,
            node_name,
            name: custom_action_name,
            param: custom_action_param,
            reco_id,
            box_rect,
        };
        (self.func)(context, &args)
    }
}

// === Trampolines ===

pub(crate) unsafe extern "C" fn custom_action_trampoline(
    context: *mut sys::MaaContext,
    task_id: sys::MaaTaskId,
    node_name: *const std::os::raw::c_char,
    custom_action_name: *const std::os::raw::c_char,
    custom_action_param: *const std::os::raw::c_char,
    reco_id: sys::MaaRecoId,
    box_: *const sys::MaaRect,
    trans_arg: *mut std::os::raw::c_void,
) -> sys::MaaBool {
    if trans_arg.is_null() {
        return 0; // Failure
    }

    let action = unsafe { &*(trans_arg as *mut Box<dyn CustomAction>) };

    let ctx = match unsafe { Context::from_raw(context) } {
        Some(c) => c,
        None => return 0,
    };

    let get_str = |ptr: *const std::os::raw::c_char| -> &str {
        if ptr.is_null() {
            ""
        } else {
            unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("") }
        }
    };

    let node = get_str(node_name);
    let name = get_str(custom_action_name);
    let param = get_str(custom_action_param);

    let rect = if !box_.is_null() {
        crate::buffer::MaaRectBuffer::from_handle(box_ as *mut sys::MaaRect)
            .map(|buf| buf.get())
            .unwrap_or(common::Rect::default())
    } else {
        common::Rect::default()
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        action.run(&ctx, task_id, node, name, param, reco_id, &rect)
    }));

    match result {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => {
            eprintln!("MaaFramework Rust Binding: Panic caught in custom action callback");
            0
        }
    }
}

pub(crate) unsafe extern "C" fn custom_recognition_trampoline(
    context: *mut sys::MaaContext,
    task_id: sys::MaaTaskId,
    node_name: *const std::os::raw::c_char,
    custom_recognition_name: *const std::os::raw::c_char,
    custom_recognition_param: *const std::os::raw::c_char,
    image: *const sys::MaaImageBuffer,
    roi: *const sys::MaaRect,
    trans_arg: *mut std::os::raw::c_void,
    out_box: *mut sys::MaaRect,
    out_detail: *mut sys::MaaStringBuffer,
) -> sys::MaaBool {
    if trans_arg.is_null() {
        return 0;
    }
    let reco = unsafe { &*(trans_arg as *mut Box<dyn CustomRecognition>) };

    let ctx = match unsafe { Context::from_raw(context) } {
        Some(c) => c,
        None => return 0,
    };

    let get_str = |ptr: *const std::os::raw::c_char| -> &str {
        if ptr.is_null() {
            ""
        } else {
            unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("") }
        }
    };

    let node = get_str(node_name);
    let name = get_str(custom_recognition_name);
    let param = get_str(custom_recognition_param);

    // Image logic: the pointer passed by C is valid for the duration of the call.
    // We wrap it safely without taking ownership.
    let img_buf = crate::buffer::MaaImageBuffer::from_handle(image as *mut sys::MaaImageBuffer);
    if img_buf.is_none() {
        return 0;
    }
    let img_buf = img_buf.unwrap();

    // Rect logic
    let roi_rect = if !roi.is_null() {
        crate::buffer::MaaRectBuffer::from_handle(roi as *mut sys::MaaRect)
            .map(|buf| buf.get())
            .unwrap_or(common::Rect::default())
    } else {
        common::Rect::default()
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        reco.analyze(&ctx, task_id, node, name, param, &img_buf, &roi_rect)
    }));

    match result {
        Ok(Some((res_rect, res_detail))) => {
            if !out_box.is_null() {
                if let Some(mut out_rect_buf) = crate::buffer::MaaRectBuffer::from_handle(out_box) {
                    let _ = out_rect_buf.set(&res_rect);
                }
            }
            if !out_detail.is_null() {
                if let Some(mut out_str_buf) =
                    crate::buffer::MaaStringBuffer::from_handle(out_detail)
                {
                    let _ = out_str_buf.set(&res_detail);
                }
            }
            1
        }
        Ok(None) => 0,
        Err(_) => {
            eprintln!("MaaFramework Rust Binding: Panic caught in custom recognition callback");
            0
        }
    }
}

// === Resource Extension impls ===

impl Resource {
    /// Register a custom action implementation.
    ///
    /// The action will be kept alive as long as it is registered in the Resource.
    /// Re-registering with the same name will drop the previous implementation.
    pub fn register_custom_action(
        &self,
        name: &str,
        action: Box<dyn CustomAction>,
    ) -> MaaResult<()> {
        let c_name = CString::new(name)?;
        let action_ptr = Box::into_raw(Box::new(action));
        let action_ptr_void = action_ptr as *mut c_void;

        unsafe {
            let ret = sys::MaaResourceRegisterCustomAction(
                self.raw(),
                c_name.as_ptr(),
                Some(custom_action_trampoline),
                action_ptr_void,
            );
            if ret == 0 {
                let _ = Box::from_raw(action_ptr);
                return Err(MaaError::FrameworkError(0));
            }
        }

        let mut map = self.custom_actions().lock().unwrap();
        if let Some(old_ptr) = map.insert(name.to_string(), action_ptr as usize) {
            unsafe {
                let _ = Box::from_raw(old_ptr as *mut Box<dyn CustomAction>);
            }
        }

        Ok(())
    }

    /// Register a custom recognition implementation.
    ///
    /// The recognizer will be kept alive as long as it is registered in the Resource.
    /// Re-registering with the same name will drop the previous implementation.
    pub fn register_custom_recognition(
        &self,
        name: &str,
        reco: Box<dyn CustomRecognition>,
    ) -> MaaResult<()> {
        let c_name = CString::new(name)?;
        let reco_ptr = Box::into_raw(Box::new(reco));
        let reco_ptr_void = reco_ptr as *mut c_void;

        unsafe {
            let ret = sys::MaaResourceRegisterCustomRecognition(
                self.raw(),
                c_name.as_ptr(),
                Some(custom_recognition_trampoline),
                reco_ptr_void,
            );
            if ret == 0 {
                let _ = Box::from_raw(reco_ptr);
                return Err(MaaError::FrameworkError(0));
            }
        }

        let mut map = self.custom_recognitions().lock().unwrap();
        if let Some(old_ptr) = map.insert(name.to_string(), reco_ptr as usize) {
            unsafe {
                let _ = Box::from_raw(old_ptr as *mut Box<dyn CustomRecognition>);
            }
        }

        Ok(())
    }
}
