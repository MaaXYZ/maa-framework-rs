use std::ffi::CString;
use std::ptr::NonNull;

use crate::{common, sys, MaaError, MaaResult};

/// Represents the execution context of a task.
///
/// This struct provides an interface for interacting with the current task's runtime state.
/// Capabilities include:
/// - Executing sub-tasks.
/// - Overriding pipeline configurations dynamically.
/// - Performing direct recognition and actions.
/// - Managing node hit counts and control flow anchors.
///
/// # Safety
///
/// `Context` is a wrapper around a non-owning pointer (`MaaContext`).
/// The underlying resources are managed by the `Tasker`. Users must ensure the `Context`
/// does not outlive the validity of the underlying task or callback scope.
pub struct Context {
    handle: NonNull<sys::MaaContext>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("handle", &self.handle)
            .finish()
    }
}

impl Context {
    pub(crate) unsafe fn from_raw(ptr: *mut sys::MaaContext) -> Option<Self> {
        NonNull::new(ptr).map(|handle| Self { handle })
    }

    /// Submits a new task for execution.
    ///
    /// # Arguments
    ///
    /// * `entry` - The name of the task entry point.
    /// * `pipeline_override` - A JSON string specifying pipeline parameter overrides.
    ///
    /// # Returns
    ///
    /// Returns the job ID (`MaaId`) associated with the submitted task.
    pub fn run_task(&self, entry: &str, pipeline_override: &str) -> MaaResult<i64> {
        let c_entry = CString::new(entry)?;
        let c_pipeline = CString::new(pipeline_override)?;
        let id = unsafe {
            sys::MaaContextRunTask(self.handle.as_ptr(), c_entry.as_ptr(), c_pipeline.as_ptr())
        };
        Ok(id)
    }

    /// Overrides pipeline parameters for the current context.
    ///
    /// # Arguments
    ///
    /// * `override_json` - A JSON string containing the parameters to override.
    pub fn override_pipeline(&self, override_json: &str) -> MaaResult<()> {
        let c_json = CString::new(override_json)?;
        let ret = unsafe { sys::MaaContextOverridePipeline(self.handle.as_ptr(), c_json.as_ptr()) };
        common::check_bool(ret)
    }

    /// Returns the underlying raw `MaaContext` pointer.
    pub fn raw(&self) -> *mut sys::MaaContext {
        self.handle.as_ptr()
    }

    /// Runs a specific recognition task with an input image.
    ///
    /// # Arguments
    ///
    /// * `entry` - The task entry name.
    /// * `pipeline_override` - A JSON string for parameter overrides.
    /// * `image` - The input image buffer.
    ///
    /// # Returns
    ///
    /// Returns the job ID associated with the recognition task.
    pub fn run_recognition(
        &self,
        entry: &str,
        pipeline_override: &str,
        image: &crate::buffer::MaaImageBuffer,
    ) -> MaaResult<i64> {
        let c_entry = CString::new(entry)?;
        let c_pipeline = CString::new(pipeline_override)?;
        let id = unsafe {
            sys::MaaContextRunRecognition(
                self.handle.as_ptr(),
                c_entry.as_ptr(),
                c_pipeline.as_ptr(),
                image.as_ptr(),
            )
        };
        Ok(id)
    }

    /// Performs a direct recognition operation.
    ///
    /// This executes a specific recognition algorithm immediately, bypassing the pipeline structure.
    ///
    /// # Arguments
    ///
    /// * `reco_type` - The specific recognition algorithm type (e.g., "TemplateMatch", "OCR").
    /// * `reco_param` - A JSON string containing the recognition parameters.
    /// * `image` - The image buffer to perform recognition on.
    ///
    /// # Returns
    ///
    /// Returns `Some(RecognitionDetail)` if successful, or `None` if the operation failed
    /// to initiate or yielded no result.
    pub fn run_recognition_direct(
        &self,
        reco_type: &str,
        reco_param: &str,
        image: &crate::buffer::MaaImageBuffer,
    ) -> MaaResult<Option<crate::common::RecognitionDetail>> {
        let c_type = CString::new(reco_type)?;
        let c_param = CString::new(reco_param)?;
        let id = unsafe {
            sys::MaaContextRunRecognitionDirect(
                self.handle.as_ptr(),
                c_type.as_ptr(),
                c_param.as_ptr(),
                image.as_ptr(),
            )
        };

        if id == 0 {
            return Ok(None);
        }

        let tasker_ptr = self.tasker_handle();
        crate::tasker::Tasker::fetch_recognition_detail(tasker_ptr, id)
    }

    /// Runs a specific action with context parameters.
    ///
    /// # Arguments
    ///
    /// * `entry` - The task entry name.
    /// * `pipeline_override` - A JSON string for parameter overrides.
    /// * `box_rect` - The target region for the action.
    /// * `reco_detail` - A string containing details from a previous recognition step.
    ///
    /// # Returns
    ///
    /// Returns the job ID associated with the action.
    pub fn run_action(
        &self,
        entry: &str,
        pipeline_override: &str,
        box_rect: &common::Rect,
        reco_detail: &str,
    ) -> MaaResult<i64> {
        let c_entry = CString::new(entry)?;
        let c_pipeline = CString::new(pipeline_override)?;
        let c_detail = CString::new(reco_detail)?;
        let maa_rect = sys::MaaRect {
            x: box_rect.x,
            y: box_rect.y,
            width: box_rect.width,
            height: box_rect.height,
        };
        let id = unsafe {
            sys::MaaContextRunAction(
                self.handle.as_ptr(),
                c_entry.as_ptr(),
                c_pipeline.as_ptr(),
                &maa_rect,
                c_detail.as_ptr(),
            )
        };
        Ok(id)
    }

    /// Performs a direct action operation.
    ///
    /// This executes a specific action immediately, bypassing the pipeline structure.
    ///
    /// # Arguments
    ///
    /// * `action_type` - The type of action to perform (e.g., "Click", "Swipe").
    /// * `action_param` - A JSON string containing the action parameters.
    /// * `box_rect` - The target region for the action (e.g., derived from recognition results).
    /// * `reco_detail` - Contextual details from a previous recognition step.
    ///
    /// # Returns
    ///
    /// Returns `Some(ActionDetail)` on success, or `None` if the operation failed.
    pub fn run_action_direct(
        &self,
        action_type: &str,
        action_param: &str,
        box_rect: &common::Rect,
        reco_detail: &str,
    ) -> MaaResult<Option<crate::common::ActionDetail>> {
        let c_type = CString::new(action_type)?;
        let c_param = CString::new(action_param)?;
        let c_detail = CString::new(reco_detail)?;
        let maa_rect = sys::MaaRect {
            x: box_rect.x,
            y: box_rect.y,
            width: box_rect.width,
            height: box_rect.height,
        };

        let id = unsafe {
            sys::MaaContextRunActionDirect(
                self.handle.as_ptr(),
                c_type.as_ptr(),
                c_param.as_ptr(),
                &maa_rect,
                c_detail.as_ptr(),
            )
        };

        if id == 0 {
            return Ok(None);
        }

        let tasker_ptr = self.tasker_handle();
        crate::tasker::Tasker::fetch_action_detail(tasker_ptr, id)
    }

    /// Overrides the execution list for a specific node.
    ///
    /// # Arguments
    ///
    /// * `node_name` - The name of the target node.
    /// * `next_list` - A slice of strings representing the new next list.
    ///   Supports special signals like `[JumpBack]` and `[Anchor]`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the override was successful.
    /// * `Ok(false)` if the operation failed.
    pub fn override_next(&self, node_name: &str, next_list: &[&str]) -> MaaResult<bool> {
        let c_name = CString::new(node_name)?;
        let list_buf = crate::buffer::MaaStringListBuffer::new()?;
        list_buf.set(next_list)?;

        let ret = unsafe {
            sys::MaaContextOverrideNext(self.handle.as_ptr(), c_name.as_ptr(), list_buf.as_ptr())
        };
        Ok(ret != 0)
    }

    /// Retrieves data associated with a specific node.
    ///
    /// # Arguments
    ///
    /// * `node_name` - The name of the node.
    ///
    /// # Returns
    ///
    /// Returns the node data as a `String` if available, or `None` otherwise.
    pub fn get_node_data(&self, node_name: &str) -> MaaResult<Option<String>> {
        let c_name = CString::new(node_name)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaContextGetNodeData(self.handle.as_ptr(), c_name.as_ptr(), buffer.as_ptr())
        };
        if ret != 0 {
            Ok(Some(buffer.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Retrieves and deserializes the object associated with a node.
    ///
    /// This is a convenience wrapper around `get_node_data` that parses the result into `PipelineData`.
    ///
    /// # Arguments
    ///
    /// * `node_name` - The name of the node.
    ///
    /// # Errors
    ///
    /// Returns `MaaError::InvalidConfig` if the data cannot be parsed.
    pub fn get_node_object(
        &self,
        node_name: &str,
    ) -> MaaResult<Option<crate::pipeline::PipelineData>> {
        if let Some(json_str) = self.get_node_data(node_name)? {
            let data: crate::pipeline::PipelineData =
                serde_json::from_str(&json_str).map_err(|e| {
                    MaaError::InvalidConfig(format!("Failed to parse pipeline data: {}", e))
                })?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Returns the ID of the current task.
    pub fn task_id(&self) -> common::MaaId {
        unsafe { sys::MaaContextGetTaskId(self.handle.as_ptr()) }
    }

    /// Associates an anchor with a specific node.
    ///
    /// # Arguments
    ///
    /// * `anchor_name` - The name of the anchor.
    /// * `node_name` - The name of the target node.
    pub fn set_anchor(&self, anchor_name: &str, node_name: &str) -> MaaResult<()> {
        let c_anchor = CString::new(anchor_name)?;
        let c_node = CString::new(node_name)?;
        let ret = unsafe {
            sys::MaaContextSetAnchor(self.handle.as_ptr(), c_anchor.as_ptr(), c_node.as_ptr())
        };
        common::check_bool(ret)
    }

    /// Retrieves the node name associated with an anchor.
    ///
    /// # Arguments
    ///
    /// * `anchor_name` - The name of the anchor.
    ///
    /// # Returns
    ///
    /// Returns the node name as a `String` if the anchor exists.
    pub fn get_anchor(&self, anchor_name: &str) -> MaaResult<Option<String>> {
        let c_anchor = CString::new(anchor_name)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaContextGetAnchor(self.handle.as_ptr(), c_anchor.as_ptr(), buffer.as_ptr())
        };
        if ret != 0 {
            Ok(Some(buffer.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Retrieves the hit count for a specific node.
    pub fn get_hit_count(&self, node_name: &str) -> MaaResult<u64> {
        let c_name = CString::new(node_name)?;
        let mut count: u64 = 0;
        let ret = unsafe {
            sys::MaaContextGetHitCount(self.handle.as_ptr(), c_name.as_ptr(), &mut count)
        };
        if ret != 0 {
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Resets the hit count for a specific node to zero.
    pub fn clear_hit_count(&self, node_name: &str) -> MaaResult<()> {
        let c_name = CString::new(node_name)?;
        let ret = unsafe { sys::MaaContextClearHitCount(self.handle.as_ptr(), c_name.as_ptr()) };
        common::check_bool(ret)
    }

    /// Creates a clone of the current context.
    ///
    /// The new context can be used for independent execution threads, preventing
    /// state interference with the original context.
    pub fn clone_context(&self) -> MaaResult<Self> {
        let cloned = unsafe { sys::MaaContextClone(self.handle.as_ptr()) };
        NonNull::new(cloned)
            .map(|handle| Self { handle })
            .ok_or(crate::MaaError::NullPointer)
    }

    /// Returns the raw handle to the associated `Tasker` instance.
    ///
    /// # Safety
    ///
    /// The returned pointer is owned by the framework. The caller must not
    /// attempt to destroy or free it.
    pub fn tasker_handle(&self) -> *mut sys::MaaTasker {
        unsafe { sys::MaaContextGetTasker(self.handle.as_ptr()) }
    }

    /// Overrides a global image resource with a provided buffer.
    ///
    /// # Arguments
    ///
    /// * `image_name` - The identifier of the image to override.
    /// * `image` - The new image data buffer.
    pub fn override_image(
        &self,
        image_name: &str,
        image: &crate::buffer::MaaImageBuffer,
    ) -> MaaResult<()> {
        let c_name = CString::new(image_name)?;
        let ret = unsafe {
            sys::MaaContextOverrideImage(self.handle.as_ptr(), c_name.as_ptr(), image.as_ptr())
        };
        common::check_bool(ret)
    }

    /// Retrieves a job handle representing the current task's execution state.
    ///
    /// This allows the caller to query the task's status or wait for its completion details.
    pub fn get_task_job(&self) -> crate::job::JobWithResult<common::TaskDetail> {
        let task_id = self.task_id();
        let tasker_raw = self.tasker_handle() as usize;

        let status_fn: crate::job::StatusFn = Box::new(move |job_id| {
            let ptr = tasker_raw as *mut sys::MaaTasker;
            common::MaaStatus(unsafe { sys::MaaTaskerStatus(ptr, job_id) })
        });

        let wait_fn: crate::job::WaitFn = Box::new(move |job_id| {
            let ptr = tasker_raw as *mut sys::MaaTasker;
            common::MaaStatus(unsafe { sys::MaaTaskerWait(ptr, job_id) })
        });

        let get_fn = move |tid: common::MaaId| -> MaaResult<Option<common::TaskDetail>> {
            let ptr = tasker_raw as *mut sys::MaaTasker;
            crate::tasker::Tasker::fetch_task_detail(ptr, tid)
        };

        crate::job::JobWithResult::new(task_id, status_fn, wait_fn, get_fn)
    }

    /// Resets hit counts for all nodes in the context.
    pub fn clear_all_hit_counts(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaContextClearHitCount(self.handle.as_ptr(), std::ptr::null()) };
        common::check_bool(ret)
    }
}
