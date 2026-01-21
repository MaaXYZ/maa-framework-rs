use crate::resource::Resource;
use crate::{common, sys, MaaError, MaaResult};
use std::ptr::NonNull;

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;

/// Task manager for executing pipelines.
///
/// Tasker is the central component that coordinates:
/// - Resource binding (images, models)
/// - Controller binding (device connection)
/// - Task execution (pipelines)
/// - Event handling (callbacks)
use std::sync::Arc;

struct TaskerInner {
    handle: NonNull<sys::MaaTasker>,
    owns_handle: bool,
    callbacks: Mutex<HashMap<sys::MaaSinkId, usize>>, // Store pointer address
    event_sinks: Mutex<HashMap<sys::MaaSinkId, usize>>,
    resource: Mutex<Option<Resource>>,
    controller: Mutex<Option<crate::controller::Controller>>,
}

unsafe impl Send for TaskerInner {}
unsafe impl Sync for TaskerInner {}

/// Task manager for executing pipelines.
///
/// Tasker is the central component that coordinates:
/// - Resource binding (images, models)
/// - Controller binding (device connection)
/// - Task execution (pipelines)
/// - Event handling (callbacks)
#[derive(Clone)]
pub struct Tasker {
    inner: Arc<TaskerInner>,
}

unsafe impl Send for Tasker {}
unsafe impl Sync for Tasker {}

impl Tasker {
    pub fn new() -> MaaResult<Self> {
        let handle = unsafe { sys::MaaTaskerCreate() };
        if let Some(ptr) = NonNull::new(handle) {
            Ok(Self {
                inner: Arc::new(TaskerInner {
                    handle: ptr,
                    owns_handle: true,
                    callbacks: Mutex::new(HashMap::new()),
                    event_sinks: Mutex::new(HashMap::new()),
                    resource: Mutex::new(None),
                    controller: Mutex::new(None),
                }),
            })
        } else {
            Err(MaaError::NullPointer)
        }
    }

    /// Create a Tasker from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid.
    /// If `owns` is true, the Tasker will destroy the handle when dropped.
    pub unsafe fn from_raw(ptr: *mut sys::MaaTasker, owns: bool) -> MaaResult<Self> {
        if let Some(handle) = NonNull::new(ptr) {
            Ok(Self {
                inner: Arc::new(TaskerInner {
                    handle,
                    owns_handle: owns,
                    callbacks: Mutex::new(HashMap::new()),
                    event_sinks: Mutex::new(HashMap::new()),
                    resource: Mutex::new(None),
                    controller: Mutex::new(None),
                }),
            })
        } else {
            Err(MaaError::NullPointer)
        }
    }

    pub fn bind_resource(&self, res: &Resource) -> MaaResult<()> {
        let ret = unsafe { sys::MaaTaskerBindResource(self.inner.handle.as_ptr(), res.raw()) };
        common::check_bool(ret)?;
        *self.inner.resource.lock().unwrap() = Some(res.clone());
        Ok(())
    }

    pub fn bind_controller(&self, ctrl: &crate::controller::Controller) -> MaaResult<()> {
        let ret = unsafe { sys::MaaTaskerBindController(self.inner.handle.as_ptr(), ctrl.raw()) };
        common::check_bool(ret)?;
        *self.inner.controller.lock().unwrap() = Some(ctrl.clone());
        Ok(())
    }

    pub fn get_recognition_detail(
        &self,
        reco_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::RecognitionDetail>> {
        Self::fetch_recognition_detail(self.inner.handle.as_ptr(), reco_id)
    }

    pub fn get_action_detail(
        &self,
        act_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::ActionDetail>> {
        Self::fetch_action_detail(self.inner.handle.as_ptr(), act_id)
    }

    pub fn get_node_detail(
        &self,
        node_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::NodeDetail>> {
        Self::fetch_node_detail(self.inner.handle.as_ptr(), node_id)
    }

    pub fn get_task_detail(
        &self,
        task_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::TaskDetail>> {
        Self::fetch_task_detail(self.inner.handle.as_ptr(), task_id)
    }

    // Static helpers for fetching details (used by both methods and jobs)

    pub(crate) fn fetch_recognition_detail(
        handle: *mut sys::MaaTasker,
        reco_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::RecognitionDetail>> {
        let node_name = crate::buffer::MaaStringBuffer::new()?;
        let algorithm = crate::buffer::MaaStringBuffer::new()?;
        let mut hit = 0;
        let mut box_rect = sys::MaaRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        let detail = crate::buffer::MaaStringBuffer::new()?;
        let raw = crate::buffer::MaaImageBuffer::new()?;
        let draws = crate::buffer::MaaImageListBuffer::new()?;

        let ret = unsafe {
            sys::MaaTaskerGetRecognitionDetail(
                handle,
                reco_id,
                node_name.raw(),
                algorithm.raw(),
                &mut hit,
                &mut box_rect,
                detail.raw(),
                raw.raw(),
                draws.raw(),
            )
        };

        if ret == 0 {
            return Ok(None);
        }

        let algorithm_str = algorithm.as_str().to_string();
        let algorithm_enum = crate::common::AlgorithmEnum::from(algorithm_str);
        let detail_val: serde_json::Value =
            serde_json::from_str(detail.as_str()).unwrap_or(serde_json::Value::Null);

        let mut sub_details = Vec::new();

        if matches!(
            algorithm_enum,
            crate::common::AlgorithmEnum::And | crate::common::AlgorithmEnum::Or
        ) {
            // Logic A: Recursive parsing for And/Or
            if let Some(arr) = detail_val.as_array() {
                for item in arr {
                    if let Some(sub_id) = item.get("reco_id").and_then(|v| v.as_i64()) {
                        if let Ok(Some(sub)) = Self::fetch_recognition_detail(handle, sub_id) {
                            sub_details.push(sub);
                        }
                    }
                }
            }
        }

        Ok(Some(crate::common::RecognitionDetail {
            node_name: node_name.as_str().to_string(),
            algorithm: algorithm_enum,
            hit: hit != 0,
            box_rect: crate::common::Rect::from(box_rect),
            detail: detail_val,
            raw_image: raw.to_vec(),
            draw_images: draws.to_vec_of_vec(),
            sub_details,
        }))
    }

    pub(crate) fn fetch_action_detail(
        handle: *mut sys::MaaTasker,
        act_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::ActionDetail>> {
        let node_name = crate::buffer::MaaStringBuffer::new()?;
        let action = crate::buffer::MaaStringBuffer::new()?;
        let mut result_box = sys::MaaRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        let mut success = 0;
        let detail = crate::buffer::MaaStringBuffer::new()?;

        let ret = unsafe {
            sys::MaaTaskerGetActionDetail(
                handle,
                act_id,
                node_name.raw(),
                action.raw(),
                &mut result_box,
                &mut success,
                detail.raw(),
            )
        };

        if ret == 0 {
            return Ok(None);
        }

        Ok(Some(crate::common::ActionDetail {
            node_name: node_name.as_str().to_string(),
            action: crate::common::ActionEnum::from(action.as_str().to_string()),
            box_rect: crate::common::Rect::from(result_box),
            success: success != 0,
            detail: serde_json::from_str(detail.as_str()).unwrap_or(serde_json::Value::Null),
        }))
    }

    pub(crate) fn fetch_node_detail(
        handle: *mut sys::MaaTasker,
        node_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::NodeDetail>> {
        let node_name = crate::buffer::MaaStringBuffer::new()?;
        let mut reco_id = 0;
        let mut act_id = 0;
        let mut completed = 0;

        let ret = unsafe {
            sys::MaaTaskerGetNodeDetail(
                handle,
                node_id,
                node_name.raw(),
                &mut reco_id,
                &mut act_id,
                &mut completed,
            )
        };

        if ret == 0 {
            return Ok(None);
        }

        // Logic A/B: Hydration implies we might want reco/act details in NodeDetail?
        // common::NodeDetail only has ids. Python wraps calls.

        let recognition = if reco_id > 0 {
            Self::fetch_recognition_detail(handle, reco_id)?
        } else {
            None
        };

        let action = if act_id > 0 {
            Self::fetch_action_detail(handle, act_id)?
        } else {
            None
        };

        Ok(Some(crate::common::NodeDetail {
            node_name: node_name.as_str().to_string(),
            reco_id,
            act_id,
            completed: completed != 0,
            recognition,
            action,
        }))
    }

    pub(crate) fn fetch_task_detail(
        handle: *mut sys::MaaTasker,
        task_id: crate::common::MaaId,
    ) -> MaaResult<Option<crate::common::TaskDetail>> {
        let entry = crate::buffer::MaaStringBuffer::new()?;
        let mut node_id_list_size: sys::MaaSize = 0;
        let mut status: sys::MaaStatus = 0;

        let ret = unsafe {
            sys::MaaTaskerGetTaskDetail(
                handle,
                task_id,
                entry.raw(),
                std::ptr::null_mut(),
                &mut node_id_list_size,
                &mut status,
            )
        };

        if ret == 0 {
            return Ok(None);
        }

        let mut node_id_list = vec![0; node_id_list_size as usize];
        let ret = unsafe {
            sys::MaaTaskerGetTaskDetail(
                handle,
                task_id,
                entry.raw(),
                node_id_list.as_mut_ptr(),
                &mut node_id_list_size,
                &mut status,
            )
        };

        if ret == 0 {
            return Ok(None);
        }

        // Logic B: Hydrate nodes
        let mut nodes = Vec::with_capacity(node_id_list.len());
        for &nid in &node_id_list {
            nodes.push(Self::fetch_node_detail(handle, nid)?);
        }

        Ok(Some(crate::common::TaskDetail {
            entry: entry.as_str().to_string(),
            node_id_list,
            status: crate::common::MaaStatus(status as i32),
            nodes,
        }))
    }

    pub fn post_task(
        &self,
        entry: &str,
        pipeline_override: &str,
    ) -> MaaResult<crate::job::TaskJob<crate::common::TaskDetail>> {
        let c_entry = std::ffi::CString::new(entry)?;
        let c_pipeline = std::ffi::CString::new(pipeline_override)?;
        let id = unsafe {
            sys::MaaTaskerPostTask(
                self.inner.handle.as_ptr(),
                c_entry.as_ptr(),
                c_pipeline.as_ptr(),
            )
        };

        let inner = self.inner.clone();
        let status_fn: crate::job::StatusFn = Box::new(move |job_id| {
            crate::common::MaaStatus(unsafe { sys::MaaTaskerStatus(inner.handle.as_ptr(), job_id) })
        });

        let inner = self.inner.clone();
        let wait_fn: crate::job::WaitFn = Box::new(move |job_id| {
            crate::common::MaaStatus(unsafe { sys::MaaTaskerWait(inner.handle.as_ptr(), job_id) })
        });

        let inner = self.inner.clone();
        let get_fn =
            move |task_id: crate::common::MaaId| -> MaaResult<Option<crate::common::TaskDetail>> {
                Tasker::fetch_task_detail(inner.handle.as_ptr(), task_id)
            };

        // Prepare override function for TaskJob
        let inner = self.inner.clone();
        let override_fn: crate::job::OverridePipelineFn = Box::new(move |job_id, pipeline| {
            let c_pipeline = std::ffi::CString::new(pipeline)?;
            let ret = unsafe {
                sys::MaaTaskerOverridePipeline(inner.handle.as_ptr(), job_id, c_pipeline.as_ptr())
            };
            Ok(ret != 0)
        });

        Ok(crate::job::TaskJob::new(
            crate::job::JobWithResult::new(id, status_fn, wait_fn, get_fn),
            override_fn,
        ))
    }

    /// Post a task with JSON pipeline override.
    ///
    /// Convenience method that accepts `serde_json::Value` for pipeline overrides.
    ///
    /// # Example
    /// ```ignore
    /// use serde_json::json;
    /// let job = tasker.post_task_json("StartTask", &json!({
    ///     "StartTask": { "next": ["SecondTask"] }
    /// }))?;
    /// ```
    pub fn post_task_json(
        &self,
        entry: &str,
        pipeline_override: &serde_json::Value,
    ) -> MaaResult<crate::job::TaskJob<crate::common::TaskDetail>> {
        self.post_task(entry, &pipeline_override.to_string())
    }

    pub fn inited(&self) -> bool {
        unsafe { sys::MaaTaskerInited(self.inner.handle.as_ptr()) != 0 }
    }

    pub fn raw(&self) -> *mut sys::MaaTasker {
        self.inner.handle.as_ptr()
    }

    /// Add a tasker event sink callback.
    ///
    /// This registers a callback that will be invoked for all tasker events
    /// including task start, task completion, and status changes.
    ///
    /// # Arguments
    /// * `callback` - Closure that receives (message, detail_json) for each event
    ///
    /// # Returns
    /// Sink ID for later removal via `remove_sink()`
    pub fn add_sink<F>(&self, callback: F) -> MaaResult<sys::MaaSinkId>
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let (cb, arg) = crate::callback::EventCallback::new(callback);
        let id = unsafe { sys::MaaTaskerAddSink(self.inner.handle.as_ptr(), cb, arg) };
        if id > 0 {
            self.inner
                .callbacks
                .lock()
                .unwrap()
                .insert(id, arg as usize);
            Ok(id)
        } else {
            unsafe { crate::callback::EventCallback::drop_callback(arg) };
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Register a strongly-typed event sink.
    ///
    /// This method registers an implementation of the [`EventSink`](crate::event_sink::EventSink) trait
    /// to receive structured notifications from this tasker.
    ///
    /// # Arguments
    /// * `sink` - The event sink implementation (must be boxed).
    ///
    /// # Returns
    /// A `MaaSinkId` which can be used to manually remove the sink later via [`remove_sink`](Self::remove_sink).
    /// The sink will be automatically unregistered and dropped when the `Tasker` is dropped.
    pub fn add_event_sink(
        &self,
        sink: Box<dyn crate::event_sink::EventSink>,
    ) -> MaaResult<sys::MaaSinkId> {
        let handle_id = self.inner.handle.as_ptr() as crate::common::MaaId;
        let (cb, arg) = crate::callback::EventCallback::new_sink(handle_id, sink);
        let id = unsafe { sys::MaaTaskerAddSink(self.inner.handle.as_ptr(), cb, arg) };
        if id > 0 {
            self.inner
                .event_sinks
                .lock()
                .unwrap()
                .insert(id, arg as usize);
            Ok(id)
        } else {
            unsafe { crate::callback::EventCallback::drop_sink(arg) };
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Remove a tasker sink by ID.
    ///
    /// # Arguments
    /// * `sink_id` - ID returned from `add_sink()`
    pub fn remove_sink(&self, sink_id: sys::MaaSinkId) {
        unsafe { sys::MaaTaskerRemoveSink(self.inner.handle.as_ptr(), sink_id) };
        if let Some(ptr) = self.inner.callbacks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut c_void) };
        } else if let Some(ptr) = self.inner.event_sinks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut c_void) };
        }
    }

    /// Clear all tasker sinks.
    pub fn clear_sinks(&self) {
        unsafe { sys::MaaTaskerClearSinks(self.inner.handle.as_ptr()) };
        let mut callbacks = self.inner.callbacks.lock().unwrap();
        for (_, ptr) in callbacks.drain() {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut c_void) };
        }
        let mut event_sinks = self.inner.event_sinks.lock().unwrap();
        for (_, ptr) in event_sinks.drain() {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut c_void) };
        }
    }

    #[deprecated(since = "0.5.1", note = "Use add_sink() instead")]
    pub fn register_callback<F>(&self, callback: F) -> MaaResult<crate::common::MaaId>
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.add_sink(callback)
    }

    pub fn post_stop(&self) -> MaaResult<crate::common::MaaId> {
        unsafe {
            let id = sys::MaaTaskerPostStop(self.inner.handle.as_ptr());
            Ok(id)
        }
    }

    /// Check if the tasker is currently running.
    pub fn is_running(&self) -> bool {
        unsafe { sys::MaaTaskerRunning(self.inner.handle.as_ptr()) != 0 }
    }

    /// Check if the tasker is currently running (alias for `is_running`).
    pub fn running(&self) -> bool {
        self.is_running()
    }

    /// Check if the tasker is currently stopping.
    pub fn stopping(&self) -> bool {
        unsafe { sys::MaaTaskerStopping(self.inner.handle.as_ptr()) != 0 }
    }

    // === Context Sink ===

    /// Add a context event sink callback.
    /// Returns a sink ID for later removal.
    pub fn add_context_sink<F>(&self, callback: F) -> MaaResult<sys::MaaSinkId>
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let (cb, arg) = crate::callback::EventCallback::new(callback);
        let id = unsafe { sys::MaaTaskerAddContextSink(self.inner.handle.as_ptr(), cb, arg) };
        if id > 0 {
            self.inner
                .callbacks
                .lock()
                .unwrap()
                .insert(id, arg as usize);
            Ok(id)
        } else {
            unsafe { crate::callback::EventCallback::drop_callback(arg) };
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Register a strongly-typed context event sink.
    ///
    /// This receives detailed execution events like Node.Recognition, Node.Action, etc.
    ///
    /// # Arguments
    /// * `sink` - The event sink implementation (must be boxed).
    ///
    /// # Returns
    /// A `MaaSinkId` which can be used to manually remove the sink later via [`remove_context_sink`](Self::remove_context_sink).
    pub fn add_context_event_sink(
        &self,
        sink: Box<dyn crate::event_sink::EventSink>,
    ) -> MaaResult<sys::MaaSinkId> {
        let handle_id = self.inner.handle.as_ptr() as crate::common::MaaId;
        let (cb, arg) = crate::callback::EventCallback::new_sink(handle_id, sink);
        let id = unsafe { sys::MaaTaskerAddContextSink(self.inner.handle.as_ptr(), cb, arg) };
        if id > 0 {
            self.inner
                .event_sinks
                .lock()
                .unwrap()
                .insert(id, arg as usize);
            Ok(id)
        } else {
            unsafe { crate::callback::EventCallback::drop_sink(arg) };
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Remove a context sink by ID.
    pub fn remove_context_sink(&self, sink_id: sys::MaaSinkId) {
        unsafe { sys::MaaTaskerRemoveContextSink(self.inner.handle.as_ptr(), sink_id) };
        if let Some(ptr) = self.inner.callbacks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut c_void) };
        } else if let Some(ptr) = self.inner.event_sinks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut c_void) };
        }
    }

    /// Clear all context sinks.
    pub fn clear_context_sinks(&self) {
        unsafe { sys::MaaTaskerClearContextSinks(self.inner.handle.as_ptr()) };
        // Note: callbacks registered via add_context_sink will be cleaned up
    }

    pub fn clear_cache(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaTaskerClearCache(self.inner.handle.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Override pipeline configuration for a specific task.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to update.
    /// * `pipeline_override` - The JSON string containing the new configuration.
    pub fn override_pipeline(
        &self,
        task_id: crate::common::MaaId,
        pipeline_override: &str,
    ) -> MaaResult<bool> {
        let c_pipeline = std::ffi::CString::new(pipeline_override)?;
        let ret = unsafe {
            sys::MaaTaskerOverridePipeline(self.inner.handle.as_ptr(), task_id, c_pipeline.as_ptr())
        };
        Ok(ret != 0)
    }

    pub fn get_latest_node(&self, node_name: &str) -> MaaResult<Option<crate::common::MaaId>> {
        let c_name = std::ffi::CString::new(node_name)?;
        let mut node_id: crate::common::MaaId = 0;
        let ret = unsafe {
            sys::MaaTaskerGetLatestNode(self.inner.handle.as_ptr(), c_name.as_ptr(), &mut node_id)
        };
        if ret != 0 && node_id != 0 {
            Ok(Some(node_id))
        } else {
            Ok(None)
        }
    }

    /// Convenience method to bind both resource and controller at once.
    pub fn bind(
        &self,
        resource: &Resource,
        controller: &crate::controller::Controller,
    ) -> MaaResult<()> {
        self.bind_resource(resource)?;
        self.bind_controller(controller)
    }

    /// Get a borrowed view of the bound resource.
    ///
    /// Returns `None` if no resource is bound.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(res) = tasker.resource() {
    ///     println!("Loaded: {}", res.loaded());
    /// }
    /// ```
    pub fn resource(&self) -> Option<crate::resource::ResourceRef<'_>> {
        let ptr = unsafe { sys::MaaTaskerGetResource(self.inner.handle.as_ptr()) };
        crate::resource::ResourceRef::from_ptr(ptr)
    }

    /// Get a borrowed view of the bound controller.
    ///
    /// Returns `None` if no controller is bound.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(ctrl) = tasker.controller() {
    ///     println!("Connected: {}", ctrl.connected());
    /// }
    /// ```
    pub fn controller(&self) -> Option<crate::controller::ControllerRef<'_>> {
        let ptr = unsafe { sys::MaaTaskerGetController(self.inner.handle.as_ptr()) };
        crate::controller::ControllerRef::from_ptr(ptr)
    }

    /// Get the bound resource handle (raw pointer).
    ///
    /// Returns the raw pointer to the resource. The caller should not destroy this handle.
    pub fn resource_handle(&self) -> *mut sys::MaaResource {
        unsafe { sys::MaaTaskerGetResource(self.inner.handle.as_ptr()) }
    }

    /// Get the bound controller handle (raw pointer).
    ///
    /// Returns the raw pointer to the controller. The caller should not destroy this handle.
    pub fn controller_handle(&self) -> *mut sys::MaaController {
        unsafe { sys::MaaTaskerGetController(self.inner.handle.as_ptr()) }
    }

    /// Post a recognition task directly without executing through a pipeline.
    ///
    /// # Arguments
    /// * `reco_type` - Recognition type (e.g., "TemplateMatch", "OCR")
    /// * `reco_param` - Recognition parameters as JSON string
    /// * `image` - The image to perform recognition on
    pub fn post_recognition(
        &self,
        reco_type: &str,
        reco_param: &str,
        image: &crate::buffer::MaaImageBuffer,
    ) -> MaaResult<crate::job::TaskJob<crate::common::RecognitionDetail>> {
        let c_type = std::ffi::CString::new(reco_type)?;
        let c_param = std::ffi::CString::new(reco_param)?;
        let id = unsafe {
            sys::MaaTaskerPostRecognition(
                self.inner.handle.as_ptr(),
                c_type.as_ptr(),
                c_param.as_ptr(),
                image.raw(),
            )
        };

        let inner = self.inner.clone();
        let status_fn: crate::job::StatusFn = Box::new(move |job_id| {
            common::MaaStatus(unsafe { sys::MaaTaskerStatus(inner.handle.as_ptr(), job_id) })
        });

        let inner = self.inner.clone();
        let wait_fn: crate::job::WaitFn = Box::new(move |job_id| {
            common::MaaStatus(unsafe { sys::MaaTaskerWait(inner.handle.as_ptr(), job_id) })
        });

        let inner = self.inner.clone();
        let get_fn = move |reco_id: common::MaaId| -> MaaResult<Option<common::RecognitionDetail>> {
            Tasker::fetch_recognition_detail(inner.handle.as_ptr(), reco_id)
        };

        // Prepare override function for TaskJob
        let inner = self.inner.clone();
        let override_fn: crate::job::OverridePipelineFn = Box::new(move |job_id, pipeline| {
            let c_pipeline = std::ffi::CString::new(pipeline)?;
            let ret = unsafe {
                sys::MaaTaskerOverridePipeline(inner.handle.as_ptr(), job_id, c_pipeline.as_ptr())
            };
            Ok(ret != 0)
        });

        Ok(crate::job::TaskJob::new(
            crate::job::JobWithResult::new(id, status_fn, wait_fn, get_fn),
            override_fn,
        ))
    }

    /// Post an action task directly without executing through a pipeline.
    ///
    /// # Arguments
    /// * `action_type` - Action type (e.g., "Click", "Swipe")  
    /// * `action_param` - Action parameters as JSON string
    /// * `box_rect` - The target rectangle for the action
    /// * `reco_detail` - Recognition detail from previous recognition (can be empty)
    pub fn post_action(
        &self,
        action_type: &str,
        action_param: &str,
        box_rect: &common::Rect,
        reco_detail: &str,
    ) -> MaaResult<crate::job::TaskJob<crate::common::ActionDetail>> {
        let c_type = std::ffi::CString::new(action_type)?;
        let c_param = std::ffi::CString::new(action_param)?;
        let c_detail = std::ffi::CString::new(reco_detail)?;
        let maa_rect = sys::MaaRect {
            x: box_rect.x,
            y: box_rect.y,
            width: box_rect.width,
            height: box_rect.height,
        };

        let id = unsafe {
            sys::MaaTaskerPostAction(
                self.inner.handle.as_ptr(),
                c_type.as_ptr(),
                c_param.as_ptr(),
                &maa_rect,
                c_detail.as_ptr(),
            )
        };

        let inner = self.inner.clone();
        let status_fn: crate::job::StatusFn = Box::new(move |job_id| {
            common::MaaStatus(unsafe { sys::MaaTaskerStatus(inner.handle.as_ptr(), job_id) })
        });

        let inner = self.inner.clone();
        let wait_fn: crate::job::WaitFn = Box::new(move |job_id| {
            common::MaaStatus(unsafe { sys::MaaTaskerWait(inner.handle.as_ptr(), job_id) })
        });

        let inner = self.inner.clone();
        let get_fn = move |act_id: common::MaaId| -> MaaResult<Option<common::ActionDetail>> {
            Tasker::fetch_action_detail(inner.handle.as_ptr(), act_id)
        };

        // Prepare override function for TaskJob
        let inner = self.inner.clone();
        let override_fn: crate::job::OverridePipelineFn = Box::new(move |job_id, pipeline| {
            let c_pipeline = std::ffi::CString::new(pipeline)?;
            let ret = unsafe {
                sys::MaaTaskerOverridePipeline(inner.handle.as_ptr(), job_id, c_pipeline.as_ptr())
            };
            Ok(ret != 0)
        });

        Ok(crate::job::TaskJob::new(
            crate::job::JobWithResult::new(id, status_fn, wait_fn, get_fn),
            override_fn,
        ))
    }

    // === Global Methods ===

    /// Set the global log directory.
    pub fn set_log_dir<P: AsRef<std::path::Path>>(path: P) -> MaaResult<()> {
        let path_str = path.as_ref().to_string_lossy();
        let c_path = std::ffi::CString::new(path_str.as_ref())?;
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_LogDir as i32,
                c_path.as_ptr() as *mut _,
                c_path.as_bytes().len() as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Set whether to save debug drawings.
    pub fn set_save_draw(save: bool) -> MaaResult<()> {
        let val = if save { 1 } else { 0 };
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_SaveDraw as i32,
                &val as *const _ as *mut _,
                std::mem::size_of_val(&val) as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Set the stdout logging level.
    pub fn set_stdout_level(level: sys::MaaLoggingLevel) -> MaaResult<()> {
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_StdoutLevel as i32,
                &level as *const _ as *mut _,
                std::mem::size_of_val(&level) as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Enable or disable debug mode (raw image capture, etc).
    pub fn set_debug_mode(debug: bool) -> MaaResult<()> {
        let val = if debug { 1 } else { 0 };
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_DebugMode as i32,
                &val as *const _ as *mut _,
                std::mem::size_of_val(&val) as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Set whether to save screenshots on error.
    pub fn set_save_on_error(save: bool) -> MaaResult<()> {
        let val = if save { 1 } else { 0 };
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_SaveOnError as i32,
                &val as *const _ as *mut _,
                std::mem::size_of_val(&val) as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Set the JPEG quality for debug drawings (0-100).
    pub fn set_draw_quality(quality: i32) -> MaaResult<()> {
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_DrawQuality as i32,
                &quality as *const _ as *mut _,
                std::mem::size_of_val(&quality) as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Set the limit for recognition image cache.
    pub fn set_reco_image_cache_limit(limit: usize) -> MaaResult<()> {
        let ret = unsafe {
            sys::MaaGlobalSetOption(
                sys::MaaGlobalOptionEnum_MaaGlobalOption_RecoImageCacheLimit as i32,
                &limit as *const _ as *mut _,
                std::mem::size_of_val(&limit) as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Load a plugin from the specified path.
    pub fn load_plugin<P: AsRef<std::path::Path>>(path: P) -> MaaResult<()> {
        let path_str = path.as_ref().to_string_lossy();
        let c_path = std::ffi::CString::new(path_str.as_ref())?;
        let ret = unsafe { sys::MaaGlobalLoadPlugin(c_path.as_ptr()) };
        common::check_bool(ret)
    }
}

impl Drop for TaskerInner {
    fn drop(&mut self) {
        unsafe {
            sys::MaaTaskerClearSinks(self.handle.as_ptr());
            let mut callbacks = self.callbacks.lock().unwrap();
            for (_, ptr) in callbacks.drain() {
                crate::callback::EventCallback::drop_callback(ptr as *mut c_void);
            }
            let mut event_sinks = self.event_sinks.lock().unwrap();
            for (_, ptr) in event_sinks.drain() {
                crate::callback::EventCallback::drop_sink(ptr as *mut c_void);
            }
            if self.owns_handle {
                sys::MaaTaskerDestroy(self.handle.as_ptr())
            }
        }
    }
}
