//! Resource management for pipelines, models, and images.

use std::ffi::CString;
use std::ptr::NonNull;

use crate::{common, sys, MaaError, MaaResult};

/// Resource manager.
///
/// Handles loading and management of:
/// - Image resources
/// - OCR models
/// - Task pipelines
/// - Custom recognizers and actions
use std::sync::Arc;

struct ResourceInner {
    handle: NonNull<sys::MaaResource>,
    custom_actions: std::sync::Mutex<std::collections::HashMap<String, usize>>, // Store pointer address
    custom_recognitions: std::sync::Mutex<std::collections::HashMap<String, usize>>,
    callbacks: std::sync::Mutex<std::collections::HashMap<sys::MaaSinkId, usize>>,
    event_sinks: std::sync::Mutex<std::collections::HashMap<sys::MaaSinkId, usize>>,
}

unsafe impl Send for ResourceInner {}
unsafe impl Sync for ResourceInner {}

#[derive(Clone)]
pub struct Resource {
    inner: Arc<ResourceInner>,
}

unsafe impl Send for Resource {}
unsafe impl Sync for Resource {}

impl Resource {
    /// Create a new resource manager.
    pub fn new() -> MaaResult<Self> {
        let handle = unsafe { sys::MaaResourceCreate() };
        if let Some(ptr) = NonNull::new(handle) {
            Ok(Self {
                inner: Arc::new(ResourceInner {
                    handle: ptr,
                    custom_actions: std::sync::Mutex::new(std::collections::HashMap::new()),
                    custom_recognitions: std::sync::Mutex::new(std::collections::HashMap::new()),
                    callbacks: std::sync::Mutex::new(std::collections::HashMap::new()),
                    event_sinks: std::sync::Mutex::new(std::collections::HashMap::new()),
                }),
            })
        } else {
            Err(MaaError::NullPointer)
        }
    }

    /// Create a Resource from a raw pointer, taking ownership.
    ///
    /// # Safety
    /// The pointer must be valid and the caller transfers ownership to the Resource.
    /// The Resource will call `MaaResourceDestroy` when dropped.
    pub unsafe fn from_raw(ptr: *mut sys::MaaResource) -> MaaResult<Self> {
        if let Some(handle) = NonNull::new(ptr) {
            Ok(Self {
                inner: Arc::new(ResourceInner {
                    handle,
                    custom_actions: std::sync::Mutex::new(std::collections::HashMap::new()),
                    custom_recognitions: std::sync::Mutex::new(std::collections::HashMap::new()),
                    callbacks: std::sync::Mutex::new(std::collections::HashMap::new()),
                    event_sinks: std::sync::Mutex::new(std::collections::HashMap::new()),
                }),
            })
        } else {
            Err(MaaError::NullPointer)
        }
    }

    /// Load a resource bundle from the specified directory.
    ///
    /// The bundle should contain pipeline definitions, images, and models.
    pub fn post_bundle(&self, path: &str) -> MaaResult<crate::job::Job> {
        let c_path = CString::new(path)?;
        let id = unsafe { sys::MaaResourcePostBundle(self.inner.handle.as_ptr(), c_path.as_ptr()) };
        Ok(crate::job::Job::for_resource(self, id))
    }

    /// Check if resources have been loaded.
    pub fn loaded(&self) -> bool {
        unsafe { sys::MaaResourceLoaded(self.inner.handle.as_ptr()) != 0 }
    }

    /// Clear all loaded resources.
    pub fn clear(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaResourceClear(self.inner.handle.as_ptr()) };
        common::check_bool(ret)
    }

    /// Get the status of a loading operation.
    pub fn status(&self, id: common::MaaId) -> common::MaaStatus {
        let status = unsafe { sys::MaaResourceStatus(self.inner.handle.as_ptr(), id) };
        common::MaaStatus(status)
    }

    /// Wait for a loading operation to complete.
    pub fn wait(&self, id: common::MaaId) -> common::MaaStatus {
        let status = unsafe { sys::MaaResourceWait(self.inner.handle.as_ptr(), id) };
        common::MaaStatus(status)
    }

    /// Get the raw resource handle.
    pub fn raw(&self) -> *mut sys::MaaResource {
        self.inner.handle.as_ptr()
    }

    // === Additional resource loading ===

    /// Load an OCR model from the specified directory.
    pub fn post_ocr_model(&self, path: &str) -> MaaResult<crate::job::Job> {
        let c_path = CString::new(path)?;
        let id =
            unsafe { sys::MaaResourcePostOcrModel(self.inner.handle.as_ptr(), c_path.as_ptr()) };
        Ok(crate::job::Job::for_resource(self, id))
    }

    /// Load additional pipeline definitions from the specified directory.
    pub fn post_pipeline(&self, path: &str) -> MaaResult<crate::job::Job> {
        let c_path = CString::new(path)?;
        let id =
            unsafe { sys::MaaResourcePostPipeline(self.inner.handle.as_ptr(), c_path.as_ptr()) };
        Ok(crate::job::Job::for_resource(self, id))
    }

    /// Load image resources from the specified directory.
    pub fn post_image(&self, path: &str) -> MaaResult<crate::job::Job> {
        let c_path = CString::new(path)?;
        let id = unsafe { sys::MaaResourcePostImage(self.inner.handle.as_ptr(), c_path.as_ptr()) };
        Ok(crate::job::Job::for_resource(self, id))
    }

    // === Pipeline operations ===

    /// Override pipeline parameters with a JSON string.
    pub fn override_pipeline(&self, pipeline_override: &str) -> MaaResult<()> {
        let c_json = CString::new(pipeline_override)?;
        let ret = unsafe {
            sys::MaaResourceOverridePipeline(self.inner.handle.as_ptr(), c_json.as_ptr())
        };
        common::check_bool(ret)
    }

    /// Override pipeline with JSON value.
    ///
    /// Convenience method that accepts `serde_json::Value` for pipeline overrides.
    ///
    /// # Example
    /// ```ignore
    /// use serde_json::json;
    /// resource.override_pipeline_json(&json!({
    ///     "MyNode": { "enabled": false }
    /// }))?;
    /// ```
    pub fn override_pipeline_json(&self, pipeline_override: &serde_json::Value) -> MaaResult<()> {
        self.override_pipeline(&pipeline_override.to_string())
    }

    /// Override the next node list for a specific node.
    ///
    /// # Arguments
    /// * `node_name` - The name of the node to modify
    /// * `next_list` - The new list of next nodes
    pub fn override_next(&self, node_name: &str, next_list: &[&str]) -> MaaResult<()> {
        let c_name = CString::new(node_name)?;
        let list_buf = crate::buffer::MaaStringListBuffer::new()?;
        for item in next_list {
            list_buf.append(item)?;
        }
        let ret = unsafe {
            sys::MaaResourceOverrideNext(
                self.inner.handle.as_ptr(),
                c_name.as_ptr(),
                list_buf.raw(),
            )
        };
        common::check_bool(ret)
    }

    /// Get node data as a JSON string.
    pub fn get_node_data(&self, node_name: &str) -> MaaResult<Option<String>> {
        let c_name = CString::new(node_name)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetNodeData(self.inner.handle.as_ptr(), c_name.as_ptr(), buffer.raw())
        };
        if ret != 0 {
            Ok(Some(buffer.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Get node data as a deserialized `PipelineData` object.
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

    /// Get a list of all node names in the loaded pipelines.
    pub fn node_list(&self) -> MaaResult<Vec<String>> {
        let buffer = crate::buffer::MaaStringListBuffer::new()?;
        let ret = unsafe { sys::MaaResourceGetNodeList(self.inner.handle.as_ptr(), buffer.raw()) };
        if ret != 0 {
            Ok(buffer.to_vec())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get a hash of the loaded resources for cache validation.
    pub fn hash(&self) -> MaaResult<String> {
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe { sys::MaaResourceGetHash(self.inner.handle.as_ptr(), buffer.raw()) };
        if ret != 0 {
            Ok(buffer.to_string())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get valid default parameters for a recognition type as JSON.
    ///
    /// # Arguments
    /// * `reco_type` - The recognition type (e.g. "TemplateMatch", "OCR").
    pub fn get_default_recognition_param(
        &self,
        reco_type: &str,
    ) -> MaaResult<Option<serde_json::Value>> {
        let c_type = CString::new(reco_type)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetDefaultRecognitionParam(
                self.inner.handle.as_ptr(),
                c_type.as_ptr(),
                buffer.raw(),
            )
        };
        if ret != 0 {
            let json_str = buffer.to_string();
            let val: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                MaaError::InvalidConfig(format!("Failed to parse default params: {}", e))
            })?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    /// Get valid default parameters for an action type as JSON.
    ///
    /// # Arguments
    /// * `action_type` - The action type (e.g. "Click", "Swipe").
    pub fn get_default_action_param(
        &self,
        action_type: &str,
    ) -> MaaResult<Option<serde_json::Value>> {
        let c_type = CString::new(action_type)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetDefaultActionParam(
                self.inner.handle.as_ptr(),
                c_type.as_ptr(),
                buffer.raw(),
            )
        };
        if ret != 0 {
            let json_str = buffer.to_string();
            let val: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                MaaError::InvalidConfig(format!("Failed to parse default params: {}", e))
            })?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    // === Inference device ===

    /// Use CPU for inference (default).
    pub fn use_cpu(&self) -> MaaResult<()> {
        let mut ep: i32 =
            sys::MaaInferenceExecutionProviderEnum_MaaInferenceExecutionProvider_CPU as i32;
        let ret1 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceExecutionProvider as i32,
                &mut ep as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret1)?;

        let mut device: i32 = sys::MaaInferenceDeviceEnum_MaaInferenceDevice_CPU as i32;
        let ret2 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceDevice as i32,
                &mut device as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret2)
    }

    /// Use DirectML for GPU-accelerated inference (Windows only).
    ///
    /// # Arguments
    /// * `device_id` - GPU device index (0 for first GPU)
    pub fn use_directml(&self, device_id: i32) -> MaaResult<()> {
        let mut ep: i32 =
            sys::MaaInferenceExecutionProviderEnum_MaaInferenceExecutionProvider_DirectML as i32;
        let ret1 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceExecutionProvider as i32,
                &mut ep as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret1)?;

        let mut device = device_id;
        let ret2 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceDevice as i32,
                &mut device as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret2)
    }

    // === EventSink ===

    /// Register a raw callback to receive events.
    ///
    /// The callback receives event type and details as JSON strings.
    pub fn add_sink<F>(&self, callback: F) -> MaaResult<sys::MaaSinkId>
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let (cb_fn, cb_arg) = crate::callback::EventCallback::new(callback);
        let sink_id = unsafe { sys::MaaResourceAddSink(self.inner.handle.as_ptr(), cb_fn, cb_arg) };
        if sink_id != 0 {
            self.inner
                .callbacks
                .lock()
                .unwrap()
                .insert(sink_id, cb_arg as usize);
            Ok(sink_id)
        } else {
            unsafe { crate::callback::EventCallback::drop_callback(cb_arg) };
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Register a strongly-typed event sink.
    ///
    /// This method registers an implementation of the [`EventSink`](crate::event_sink::EventSink) trait
    /// to receive structured notifications from this resource.
    ///
    /// # Arguments
    /// * `sink` - The event sink implementation (must be boxed).
    ///
    /// # Returns
    /// A `MaaSinkId` which can be used to manually remove the sink later via [`remove_sink`](Self::remove_sink).
    /// The sink will be automatically unregistered and dropped when the `Resource` is dropped.
    pub fn add_event_sink(
        &self,
        sink: Box<dyn crate::event_sink::EventSink>,
    ) -> MaaResult<sys::MaaSinkId> {
        let handle_id = self.inner.handle.as_ptr() as crate::common::MaaId;
        let (cb, arg) = crate::callback::EventCallback::new_sink(handle_id, sink);
        let id = unsafe { sys::MaaResourceAddSink(self.inner.handle.as_ptr(), cb, arg) };
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

    /// Remove a previously registered event sink.
    pub fn remove_sink(&self, sink_id: sys::MaaSinkId) {
        unsafe { sys::MaaResourceRemoveSink(self.inner.handle.as_ptr(), sink_id) }
        if let Some(ptr) = self.inner.callbacks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut std::ffi::c_void) };
        } else if let Some(ptr) = self.inner.event_sinks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut std::ffi::c_void) };
        }
    }

    /// Remove all registered event sinks.
    pub fn clear_sinks(&self) {
        unsafe { sys::MaaResourceClearSinks(self.inner.handle.as_ptr()) }
        let mut callbacks = self.inner.callbacks.lock().unwrap();
        for (_, ptr) in callbacks.drain() {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut std::ffi::c_void) };
        }
        let mut event_sinks = self.inner.event_sinks.lock().unwrap();
        for (_, ptr) in event_sinks.drain() {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut std::ffi::c_void) };
        }
    }

    // === Image override ===

    /// Override an image resource at runtime.
    ///
    /// # Arguments
    /// * `image_name` - The name of the image to override
    /// * `image` - The new image buffer to use
    pub fn override_image(
        &self,
        image_name: &str,
        image: &crate::buffer::MaaImageBuffer,
    ) -> MaaResult<()> {
        let c_name = CString::new(image_name)?;
        let ret = unsafe {
            sys::MaaResourceOverrideImage(self.inner.handle.as_ptr(), c_name.as_ptr(), image.raw())
        };
        common::check_bool(ret)
    }

    // === Inference device (extended) ===

    /// Auto-select the best inference execution provider.
    pub fn use_auto_ep(&self) -> MaaResult<()> {
        let mut ep: i32 =
            sys::MaaInferenceExecutionProviderEnum_MaaInferenceExecutionProvider_Auto as i32;
        let ret1 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceExecutionProvider as i32,
                &mut ep as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret1)?;

        let mut device: i32 = sys::MaaInferenceDeviceEnum_MaaInferenceDevice_Auto as i32;
        let ret2 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceDevice as i32,
                &mut device as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret2)
    }

    /// Use CoreML for inference (macOS only).
    ///
    /// # Arguments
    /// * `coreml_flag` - CoreML configuration flag
    pub fn use_coreml(&self, coreml_flag: i32) -> MaaResult<()> {
        let mut ep: i32 =
            sys::MaaInferenceExecutionProviderEnum_MaaInferenceExecutionProvider_CoreML as i32;
        let ret1 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceExecutionProvider as i32,
                &mut ep as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret1)?;

        let mut device = coreml_flag;
        let ret2 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceDevice as i32,
                &mut device as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret2)
    }

    /// Use CUDA for inference (NVIDIA GPU only).
    ///
    /// # Arguments
    /// * `nvidia_gpu_id` - NVIDIA GPU device ID (typically 0 for first GPU)
    pub fn use_cuda(&self, nvidia_gpu_id: i32) -> MaaResult<()> {
        let mut ep: i32 =
            sys::MaaInferenceExecutionProviderEnum_MaaInferenceExecutionProvider_CUDA as i32;
        let ret1 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceExecutionProvider as i32,
                &mut ep as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret1)?;

        let mut device = nvidia_gpu_id;
        let ret2 = unsafe {
            sys::MaaResourceSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaResOptionEnum_MaaResOption_InferenceDevice as i32,
                &mut device as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret2)
    }

    // === Custom component management ===

    /// Unregister a custom recognition by name.
    pub fn unregister_custom_recognition(&self, name: &str) -> MaaResult<()> {
        let c_name = CString::new(name)?;
        let ret = unsafe {
            sys::MaaResourceUnregisterCustomRecognition(self.inner.handle.as_ptr(), c_name.as_ptr())
        };
        if ret != 0 {
            self.inner.custom_recognitions.lock().unwrap().remove(name);
        }
        common::check_bool(ret)
    }

    /// Unregister a custom action by name.
    pub fn unregister_custom_action(&self, name: &str) -> MaaResult<()> {
        let c_name = CString::new(name)?;
        let ret = unsafe {
            sys::MaaResourceUnregisterCustomAction(self.inner.handle.as_ptr(), c_name.as_ptr())
        };
        if ret != 0 {
            self.inner.custom_actions.lock().unwrap().remove(name);
        }
        common::check_bool(ret)
    }

    /// Get the list of registered custom recognitions.
    pub fn custom_recognition_list(&self) -> MaaResult<Vec<String>> {
        let buffer = crate::buffer::MaaStringListBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetCustomRecognitionList(self.inner.handle.as_ptr(), buffer.raw())
        };
        if ret != 0 {
            Ok(buffer.to_vec())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get the list of registered custom actions.
    pub fn custom_action_list(&self) -> MaaResult<Vec<String>> {
        let buffer = crate::buffer::MaaStringListBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetCustomActionList(self.inner.handle.as_ptr(), buffer.raw())
        };
        if ret != 0 {
            Ok(buffer.to_vec())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Clear all registered custom recognitions.
    pub fn clear_custom_recognition(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaResourceClearCustomRecognition(self.inner.handle.as_ptr()) };
        if ret != 0 {
            let mut recos = self.inner.custom_recognitions.lock().unwrap();
            for (_, ptr) in recos.drain() {
                unsafe {
                    let _ = Box::from_raw(ptr as *mut Box<dyn crate::custom::CustomRecognition>);
                }
            }
        }
        common::check_bool(ret)
    }

    /// Clear all registered custom actions.
    pub fn clear_custom_action(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaResourceClearCustomAction(self.inner.handle.as_ptr()) };
        if ret != 0 {
            let mut actions = self.inner.custom_actions.lock().unwrap();
            for (_, ptr) in actions.drain() {
                unsafe {
                    let _ = Box::from_raw(ptr as *mut Box<dyn crate::custom::CustomAction>);
                }
            }
        }
        common::check_bool(ret)
    }
    pub(crate) fn custom_recognitions(
        &self,
    ) -> &std::sync::Mutex<std::collections::HashMap<String, usize>> {
        &self.inner.custom_recognitions
    }

    pub(crate) fn custom_actions(
        &self,
    ) -> &std::sync::Mutex<std::collections::HashMap<String, usize>> {
        &self.inner.custom_actions
    }
}

impl Drop for ResourceInner {
    fn drop(&mut self) {
        unsafe {
            {
                let mut callbacks = self.callbacks.lock().unwrap();
                for (_, ptr) in callbacks.drain() {
                    let _ =
                        crate::callback::EventCallback::drop_callback(ptr as *mut std::ffi::c_void);
                }
            }
            {
                let mut event_sinks = self.event_sinks.lock().unwrap();
                for (_, ptr) in event_sinks.drain() {
                    let _ = crate::callback::EventCallback::drop_sink(ptr as *mut std::ffi::c_void);
                }
            }
            sys::MaaResourceClearSinks(self.handle.as_ptr());
            sys::MaaResourceClearCustomAction(self.handle.as_ptr());
            sys::MaaResourceClearCustomRecognition(self.handle.as_ptr());

            {
                let mut actions = self.custom_actions.lock().unwrap();
                for (_, ptr) in actions.drain() {
                    let _ = Box::from_raw(ptr as *mut Box<dyn crate::custom::CustomAction>);
                }
            }
            {
                let mut recos = self.custom_recognitions.lock().unwrap();
                for (_, ptr) in recos.drain() {
                    let _ = Box::from_raw(ptr as *mut Box<dyn crate::custom::CustomRecognition>);
                }
            }

            sys::MaaResourceDestroy(self.handle.as_ptr())
        }
    }
}

/// A borrowed reference to a Resource.
///
/// This is a non-owning view that can be used for read-only operations.
/// It does NOT call destroy when dropped and should only be used while
/// the underlying Resource is still alive.
///
/// Obtained from `Tasker::resource()`.
pub struct ResourceRef<'a> {
    handle: *mut sys::MaaResource,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ResourceRef<'a> {
    pub(crate) fn from_ptr(handle: *mut sys::MaaResource) -> Option<Self> {
        if handle.is_null() {
            None
        } else {
            Some(Self {
                handle,
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// Check if resources have been loaded.
    pub fn loaded(&self) -> bool {
        unsafe { sys::MaaResourceLoaded(self.handle) != 0 }
    }

    /// Get the status of a loading operation.
    pub fn status(&self, id: common::MaaId) -> common::MaaStatus {
        let status = unsafe { sys::MaaResourceStatus(self.handle, id) };
        common::MaaStatus(status)
    }

    /// Wait for a loading operation to complete.
    pub fn wait(&self, id: common::MaaId) -> common::MaaStatus {
        let status = unsafe { sys::MaaResourceWait(self.handle, id) };
        common::MaaStatus(status)
    }

    /// Get resource hash.
    pub fn hash(&self) -> MaaResult<String> {
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe { sys::MaaResourceGetHash(self.handle, buffer.raw()) };
        if ret != 0 {
            Ok(buffer.to_string())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get valid default parameters for a recognition type as JSON.
    pub fn get_default_recognition_param(
        &self,
        reco_type: &str,
    ) -> MaaResult<Option<serde_json::Value>> {
        let c_type = std::ffi::CString::new(reco_type)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetDefaultRecognitionParam(self.handle, c_type.as_ptr(), buffer.raw())
        };
        if ret != 0 {
            let json_str = buffer.to_string();
            let val: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                MaaError::InvalidConfig(format!("Failed to parse default params: {}", e))
            })?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    /// Get valid default parameters for an action type as JSON.
    pub fn get_default_action_param(
        &self,
        action_type: &str,
    ) -> MaaResult<Option<serde_json::Value>> {
        let c_type = std::ffi::CString::new(action_type)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaResourceGetDefaultActionParam(self.handle, c_type.as_ptr(), buffer.raw())
        };
        if ret != 0 {
            let json_str = buffer.to_string();
            let val: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                MaaError::InvalidConfig(format!("Failed to parse default params: {}", e))
            })?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    /// Get node list.
    pub fn node_list(&self) -> MaaResult<Vec<String>> {
        let buffer = crate::buffer::MaaStringListBuffer::new()?;
        let ret = unsafe { sys::MaaResourceGetNodeList(self.handle, buffer.raw()) };
        if ret != 0 {
            Ok(buffer.to_vec())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get node data as JSON string.
    pub fn get_node_data(&self, node_name: &str) -> MaaResult<Option<String>> {
        let c_name = std::ffi::CString::new(node_name)?;
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret =
            unsafe { sys::MaaResourceGetNodeData(self.handle, c_name.as_ptr(), buffer.raw()) };
        if ret != 0 {
            Ok(Some(buffer.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Get the raw handle.
    pub fn raw(&self) -> *mut sys::MaaResource {
        self.handle
    }
}
