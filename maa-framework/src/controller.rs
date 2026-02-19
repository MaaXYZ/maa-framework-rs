//! Device controller for input, screen capture, and app management.

use crate::{common, sys, MaaError, MaaResult};
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

/// Device controller interface.
///
/// Handles interaction with the target device, including:
/// - Input events (click, swipe, key press)
/// - Screen capture
/// - App management (start/stop)
/// - Connection management
///
/// See also: [`AdbControllerBuilder`] for advanced ADB configuration.
#[derive(Clone)]
pub struct Controller {
    inner: Arc<ControllerInner>,
}

struct ControllerInner {
    handle: NonNull<sys::MaaController>,
    owns_handle: bool,
    callbacks: Mutex<HashMap<sys::MaaSinkId, usize>>,
    event_sinks: Mutex<HashMap<sys::MaaSinkId, usize>>,
}

unsafe impl Send for ControllerInner {}
unsafe impl Sync for ControllerInner {}

// Controller is Send/Sync because it holds Arc<ControllerInner> which is Send/Sync
unsafe impl Send for Controller {}
unsafe impl Sync for Controller {}

impl std::fmt::Debug for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Controller")
            .field("handle", &self.inner.handle)
            .finish()
    }
}

impl Controller {
    /// Create a new ADB controller for Android device control.
    ///
    /// # Arguments
    /// * `adb_path` - Path to the ADB executable
    /// * `address` - Device address (e.g., "127.0.0.1:5555" or "emulator-5554")
    /// * `config` - JSON configuration string for advanced options
    /// * `agent_path` - Path to MaaAgent binary; pass `""` to use current directory (may return `Err` if resolution fails).
    #[cfg(feature = "adb")]
    pub fn new_adb(
        adb_path: &str,
        address: &str,
        config: &str,
        agent_path: &str,
    ) -> MaaResult<Self> {
        Self::create_adb(
            adb_path,
            address,
            sys::MaaAdbScreencapMethod_Default as sys::MaaAdbScreencapMethod,
            sys::MaaAdbInputMethod_Default as sys::MaaAdbInputMethod,
            config,
            agent_path,
        )
    }

    /// Resolves to the current directory if the string is empty; otherwise, uses it as-is. Returns Err if parsing fails.
    #[cfg(feature = "adb")]
    fn resolve_agent_path(agent_path: &str) -> MaaResult<String> {
        if !agent_path.is_empty() {
            return Ok(agent_path.to_string());
        }
        let cur = std::env::current_dir().
                           map_err(|e| {
                           MaaError::InvalidArgument(
                           format!("agent_path empty and current_dir failed: {}", e)
                        )
        })?;
        let s = cur.
                    to_str().
                    ok_or_else(|| {
                    MaaError::InvalidArgument(
                   "agent_path empty and current directory is not valid UTF-8".to_string(),
            )
        })?;
        Ok(s.to_string())
    }

    #[cfg(feature = "adb")]
    pub(crate) fn create_adb(
        adb_path: &str,
        address: &str,
        screencap_method: sys::MaaAdbScreencapMethod,
        input_method: sys::MaaAdbInputMethod,
        config: &str,
        agent_path: &str,
    ) -> MaaResult<Self> {
        let path = Self::resolve_agent_path(agent_path)?;
        let c_adb = CString::new(adb_path)?;
        let c_addr = CString::new(address)?;
        let c_cfg = CString::new(config)?;
        let c_agent = CString::new(path.as_str())?;

        let handle = unsafe {
            sys::MaaAdbControllerCreate(
                c_adb.as_ptr(),
                c_addr.as_ptr(),
                screencap_method,
                input_method,
                c_cfg.as_ptr(),
                c_agent.as_ptr(),
            )
        };

        if let Some(ptr) = NonNull::new(handle) {
            Ok(Self {
                inner: Arc::new(ControllerInner {
                    handle: ptr,
                    owns_handle: true,
                    callbacks: Mutex::new(HashMap::new()),
                    event_sinks: Mutex::new(HashMap::new()),
                }),
            })
        } else {
            Err(MaaError::FrameworkError(-1))
        }
    }

    /// Create a new Win32 controller for Windows window control.
    #[cfg(feature = "win32")]
    pub fn new_win32(
        hwnd: *mut c_void,
        screencap_method: sys::MaaWin32ScreencapMethod,
        mouse_method: sys::MaaWin32InputMethod,
        keyboard_method: sys::MaaWin32InputMethod,
    ) -> MaaResult<Self> {
        let handle = unsafe {
            sys::MaaWin32ControllerCreate(hwnd, screencap_method, mouse_method, keyboard_method)
        };

        Self::from_handle(handle)
    }

    /// Create a new PlayCover controller for iOS app control on macOS.

    pub fn new_playcover(address: &str, uuid: &str) -> MaaResult<Self> {
        let c_addr = CString::new(address)?;
        let c_uuid = CString::new(uuid)?;
        let handle = unsafe { sys::MaaPlayCoverControllerCreate(c_addr.as_ptr(), c_uuid.as_ptr()) };

        Self::from_handle(handle)
    }

    /// Create a custom controller with user-defined callbacks.
    #[cfg(feature = "custom")]
    pub fn new_custom<T: crate::custom_controller::CustomControllerCallback + 'static>(
        callback: T,
    ) -> MaaResult<Self> {
        let boxed: Box<Box<dyn crate::custom_controller::CustomControllerCallback>> =
            Box::new(Box::new(callback));
        let cb_ptr = Box::into_raw(boxed) as *mut c_void;
        let callbacks = crate::custom_controller::get_callbacks();
        let handle =
            unsafe { sys::MaaCustomControllerCreate(callbacks as *const _ as *mut _, cb_ptr) };

        NonNull::new(handle)
            .map(|ptr| Self {
                inner: Arc::new(ControllerInner {
                    handle: ptr,
                    owns_handle: true,
                    callbacks: Mutex::new(HashMap::new()),
                    event_sinks: Mutex::new(HashMap::new()),
                }),
            })
            .ok_or_else(|| {
                unsafe {
                    let _ = Box::from_raw(
                        cb_ptr as *mut Box<dyn crate::custom_controller::CustomControllerCallback>,
                    );
                }
                MaaError::FrameworkError(-1)
            })
    }

    /// Helper to create controller from raw handle.
    fn from_handle(handle: *mut sys::MaaController) -> MaaResult<Self> {
        if let Some(ptr) = NonNull::new(handle) {
            Ok(Self {
                inner: Arc::new(ControllerInner {
                    handle: ptr,
                    owns_handle: true,
                    callbacks: Mutex::new(HashMap::new()),
                    event_sinks: Mutex::new(HashMap::new()),
                }),
            })
        } else {
            Err(MaaError::FrameworkError(-1))
        }
    }

    /// Post a click action at the specified coordinates.
    pub fn post_click(&self, x: i32, y: i32) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostClick(self.inner.handle.as_ptr(), x, y) };
        Ok(id)
    }

    /// Post a screenshot capture request.
    pub fn post_screencap(&self) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostScreencap(self.inner.handle.as_ptr()) };
        Ok(id)
    }

    /// Post a click action with contact and pressure parameters.
    ///
    /// # Arguments
    /// * `x`, `y` - Click coordinates
    /// * `contact` - Contact/finger index (for multi-touch)
    /// * `pressure` - Touch pressure (1 = normal)
    pub fn post_click_v2(
        &self,
        x: i32,
        y: i32,
        contact: i32,
        pressure: i32,
    ) -> MaaResult<common::MaaId> {
        let id = unsafe {
            sys::MaaControllerPostClickV2(self.inner.handle.as_ptr(), x, y, contact, pressure)
        };
        Ok(id)
    }

    /// Post a swipe action from one point to another.
    ///
    /// # Arguments
    /// * `x1`, `y1` - Start coordinates
    /// * `x2`, `y2` - End coordinates
    /// * `duration` - Swipe duration in milliseconds
    pub fn post_swipe(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration: i32,
    ) -> MaaResult<common::MaaId> {
        let id = unsafe {
            sys::MaaControllerPostSwipe(self.inner.handle.as_ptr(), x1, y1, x2, y2, duration)
        };
        Ok(id)
    }

    /// Post a key click action.
    ///
    /// # Arguments
    /// * `keycode` - Virtual key code (ADB keycode for Android, VK for Win32)
    pub fn post_click_key(&self, keycode: i32) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostClickKey(self.inner.handle.as_ptr(), keycode) };
        Ok(id)
    }

    /// Alias for [`post_click_key`](Self::post_click_key).
    #[deprecated(note = "Use post_click_key instead")]
    pub fn post_press(&self, keycode: i32) -> MaaResult<common::MaaId> {
        self.post_click_key(keycode)
    }

    /// Post a text input action.
    ///
    /// # Arguments
    /// * `text` - Text to input
    pub fn post_input_text(&self, text: &str) -> MaaResult<common::MaaId> {
        let c_text = CString::new(text)?;
        let id =
            unsafe { sys::MaaControllerPostInputText(self.inner.handle.as_ptr(), c_text.as_ptr()) };
        Ok(id)
    }

    /// Post a shell command execution (ADB only).
    ///
    /// # Arguments
    /// * `cmd` - Shell command to execute
    /// * `timeout` - Timeout in milliseconds
    pub fn post_shell(&self, cmd: &str, timeout: i64) -> MaaResult<common::MaaId> {
        let c_cmd = CString::new(cmd)?;
        let id = unsafe {
            sys::MaaControllerPostShell(self.inner.handle.as_ptr(), c_cmd.as_ptr(), timeout)
        };
        Ok(id)
    }

    /// Post a touch down event.
    ///
    /// # Arguments
    /// * `contact` - Contact/finger index
    /// * `x`, `y` - Touch coordinates
    /// * `pressure` - Touch pressure
    pub fn post_touch_down(
        &self,
        contact: i32,
        x: i32,
        y: i32,
        pressure: i32,
    ) -> MaaResult<common::MaaId> {
        let id = unsafe {
            sys::MaaControllerPostTouchDown(self.inner.handle.as_ptr(), contact, x, y, pressure)
        };
        Ok(id)
    }

    /// Post a touch move event.
    ///
    /// # Arguments
    /// * `contact` - Contact/finger index
    /// * `x`, `y` - New touch coordinates
    /// * `pressure` - Touch pressure
    pub fn post_touch_move(
        &self,
        contact: i32,
        x: i32,
        y: i32,
        pressure: i32,
    ) -> MaaResult<common::MaaId> {
        let id = unsafe {
            sys::MaaControllerPostTouchMove(self.inner.handle.as_ptr(), contact, x, y, pressure)
        };
        Ok(id)
    }

    /// Post a touch up event.
    ///
    /// # Arguments
    /// * `contact` - Contact/finger index to release
    pub fn post_touch_up(&self, contact: i32) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostTouchUp(self.inner.handle.as_ptr(), contact) };
        Ok(id)
    }

    /// Returns the underlying raw controller handle.
    #[inline]
    pub fn raw(&self) -> *mut sys::MaaController {
        self.inner.handle.as_ptr()
    }

    // === Connection ===

    /// Post a connection request to the device.
    ///
    /// Returns a job ID that can be used with [`wait`](Self::wait) to block until connected.
    pub fn post_connection(&self) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostConnection(self.inner.handle.as_ptr()) };
        Ok(id)
    }

    /// Returns `true` if the controller is connected to the device.
    pub fn connected(&self) -> bool {
        unsafe { sys::MaaControllerConnected(self.inner.handle.as_ptr()) != 0 }
    }

    /// Gets the unique identifier (UUID) of the connected device.
    pub fn uuid(&self) -> MaaResult<String> {
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe { sys::MaaControllerGetUuid(self.inner.handle.as_ptr(), buffer.as_ptr()) };
        if ret != 0 {
            Ok(buffer.to_string())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Gets the device screen resolution as (width, height).
    pub fn resolution(&self) -> MaaResult<(i32, i32)> {
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        let ret = unsafe {
            sys::MaaControllerGetResolution(self.inner.handle.as_ptr(), &mut width, &mut height)
        };
        if ret != 0 {
            Ok((width, height))
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    // === Swipe V2 ===

    /// Post a swipe action with contact and pressure parameters.
    ///
    /// # Arguments
    /// * `x1`, `y1` - Start coordinates
    /// * `x2`, `y2` - End coordinates
    /// * `duration` - Swipe duration in milliseconds
    /// * `contact` - Contact/finger index
    /// * `pressure` - Touch pressure
    pub fn post_swipe_v2(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration: i32,
        contact: i32,
        pressure: i32,
    ) -> MaaResult<common::MaaId> {
        let id = unsafe {
            sys::MaaControllerPostSwipeV2(
                self.inner.handle.as_ptr(),
                x1,
                y1,
                x2,
                y2,
                duration,
                contact,
                pressure,
            )
        };
        Ok(id)
    }

    // === Key control ===

    /// Post a key down event.
    pub fn post_key_down(&self, keycode: i32) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostKeyDown(self.inner.handle.as_ptr(), keycode) };
        Ok(id)
    }

    /// Post a key up event.
    pub fn post_key_up(&self, keycode: i32) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostKeyUp(self.inner.handle.as_ptr(), keycode) };
        Ok(id)
    }

    // === App control ===

    /// Start an application.
    ///
    /// # Arguments
    /// * `intent` - Package name or activity (ADB), app identifier (Win32)
    pub fn post_start_app(&self, intent: &str) -> MaaResult<common::MaaId> {
        let c_intent = CString::new(intent)?;
        let id = unsafe {
            sys::MaaControllerPostStartApp(self.inner.handle.as_ptr(), c_intent.as_ptr())
        };
        Ok(id)
    }

    /// Stop an application.
    ///
    /// # Arguments
    /// * `intent` - Package name (ADB)
    pub fn post_stop_app(&self, intent: &str) -> MaaResult<common::MaaId> {
        let c_intent = CString::new(intent)?;
        let id =
            unsafe { sys::MaaControllerPostStopApp(self.inner.handle.as_ptr(), c_intent.as_ptr()) };
        Ok(id)
    }

    // === Scroll ===

    /// Post a scroll action (Win32 only).
    ///
    /// # Arguments
    /// * `dx` - Horizontal scroll delta (positive = right)
    /// * `dy` - Vertical scroll delta (positive = down)
    pub fn post_scroll(&self, dx: i32, dy: i32) -> MaaResult<common::MaaId> {
        let id = unsafe { sys::MaaControllerPostScroll(self.inner.handle.as_ptr(), dx, dy) };
        Ok(id)
    }

    // === Image ===

    /// Gets the most recently captured screenshot.
    pub fn cached_image(&self) -> MaaResult<crate::buffer::MaaImageBuffer> {
        let buffer = crate::buffer::MaaImageBuffer::new()?;
        let ret =
            unsafe { sys::MaaControllerCachedImage(self.inner.handle.as_ptr(), buffer.as_ptr()) };
        if ret != 0 {
            Ok(buffer)
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    // === Shell output ===

    /// Gets the output from the most recent shell command (ADB only).
    pub fn shell_output(&self) -> MaaResult<String> {
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe {
            sys::MaaControllerGetShellOutput(self.inner.handle.as_ptr(), buffer.as_ptr())
        };
        if ret != 0 {
            Ok(buffer.to_string())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    // === Status ===

    /// Gets the status of a controller operation.
    pub fn status(&self, ctrl_id: common::MaaId) -> common::MaaStatus {
        let s = unsafe { sys::MaaControllerStatus(self.inner.handle.as_ptr(), ctrl_id) };
        common::MaaStatus(s)
    }

    /// Blocks until a controller operation completes.
    pub fn wait(&self, ctrl_id: common::MaaId) -> common::MaaStatus {
        let s = unsafe { sys::MaaControllerWait(self.inner.handle.as_ptr(), ctrl_id) };
        common::MaaStatus(s)
    }

    // === Screenshot options ===

    /// Sets the target long side for screenshot scaling.
    pub fn set_screenshot_target_long_side(&self, long_side: i32) -> MaaResult<()> {
        let mut val = long_side;
        let ret = unsafe {
            sys::MaaControllerSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaCtrlOptionEnum_MaaCtrlOption_ScreenshotTargetLongSide as i32,
                &mut val as *mut _ as *mut c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Sets the target short side for screenshot scaling.
    pub fn set_screenshot_target_short_side(&self, short_side: i32) -> MaaResult<()> {
        let mut val = short_side;
        let ret = unsafe {
            sys::MaaControllerSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaCtrlOptionEnum_MaaCtrlOption_ScreenshotTargetShortSide as i32,
                &mut val as *mut _ as *mut c_void,
                std::mem::size_of::<i32>() as u64,
            )
        };
        common::check_bool(ret)
    }

    /// Sets whether to use raw (unscaled) screenshot resolution.
    pub fn set_screenshot_use_raw_size(&self, enable: bool) -> MaaResult<()> {
        let mut val: u8 = if enable { 1 } else { 0 };
        let ret = unsafe {
            sys::MaaControllerSetOption(
                self.inner.handle.as_ptr(),
                sys::MaaCtrlOptionEnum_MaaCtrlOption_ScreenshotUseRawSize as i32,
                &mut val as *mut _ as *mut c_void,
                std::mem::size_of::<u8>() as u64,
            )
        };
        common::check_bool(ret)
    }

    // === New controller types ===

    #[cfg(feature = "dbg")]
    pub fn new_dbg(
        read_path: &str,
        write_path: &str,
        dbg_type: sys::MaaDbgControllerType,
        config: &str,
    ) -> MaaResult<Self> {
        let c_read = CString::new(read_path)?;
        let c_write = CString::new(write_path)?;
        let c_cfg = CString::new(config)?;
        let handle = unsafe {
            sys::MaaDbgControllerCreate(c_read.as_ptr(), c_write.as_ptr(), dbg_type, c_cfg.as_ptr())
        };
        Self::from_handle(handle)
    }

    /// Create a virtual gamepad controller (Windows only).
    #[cfg(feature = "win32")]
    pub fn new_gamepad(
        hwnd: *mut c_void,
        gamepad_type: crate::common::GamepadType,
        screencap_method: crate::common::Win32ScreencapMethod,
    ) -> MaaResult<Self> {
        let handle = unsafe {
            sys::MaaGamepadControllerCreate(hwnd, gamepad_type as u64, screencap_method.bits())
        };
        Self::from_handle(handle)
    }

    // === EventSink ===

    /// Returns sink_id for later removal. Callback lifetime managed by caller.
    pub fn add_sink<F>(&self, callback: F) -> MaaResult<sys::MaaSinkId>
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let (cb_fn, cb_arg) = crate::callback::EventCallback::new(callback);
        let sink_id =
            unsafe { sys::MaaControllerAddSink(self.inner.handle.as_ptr(), cb_fn, cb_arg) };
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
    /// to receive structured notifications from this controller.
    ///
    /// # Arguments
    /// * `sink` - The event sink implementation (must be boxed).
    ///
    /// # Returns
    /// A `MaaSinkId` which can be used to manually remove the sink later via [`remove_sink`](Self::remove_sink).
    /// The sink will be automatically unregistered and dropped when the `Controller` is dropped.
    pub fn add_event_sink(
        &self,
        sink: Box<dyn crate::event_sink::EventSink>,
    ) -> MaaResult<sys::MaaSinkId> {
        let handle_id = self.inner.handle.as_ptr() as crate::common::MaaId;
        let (cb, arg) = crate::callback::EventCallback::new_sink(handle_id, sink);
        let id = unsafe { sys::MaaControllerAddSink(self.inner.handle.as_ptr(), cb, arg) };
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

    pub fn remove_sink(&self, sink_id: sys::MaaSinkId) {
        unsafe { sys::MaaControllerRemoveSink(self.inner.handle.as_ptr(), sink_id) };
        if let Some(ptr) = self.inner.callbacks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut c_void) };
        } else if let Some(ptr) = self.inner.event_sinks.lock().unwrap().remove(&sink_id) {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut c_void) };
        }
    }

    pub fn clear_sinks(&self) {
        unsafe { sys::MaaControllerClearSinks(self.inner.handle.as_ptr()) };
        let mut callbacks = self.inner.callbacks.lock().unwrap();
        for (_, ptr) in callbacks.drain() {
            unsafe { crate::callback::EventCallback::drop_callback(ptr as *mut c_void) };
        }
        let mut event_sinks = self.inner.event_sinks.lock().unwrap();
        for (_, ptr) in event_sinks.drain() {
            unsafe { crate::callback::EventCallback::drop_sink(ptr as *mut c_void) };
        }
    }
}

impl Drop for ControllerInner {
    fn drop(&mut self) {
        unsafe {
            sys::MaaControllerClearSinks(self.handle.as_ptr());
            let mut callbacks = self.callbacks.lock().unwrap();
            for (_, ptr) in callbacks.drain() {
                crate::callback::EventCallback::drop_callback(ptr as *mut c_void);
            }
            let mut event_sinks = self.event_sinks.lock().unwrap();
            for (_, ptr) in event_sinks.drain() {
                crate::callback::EventCallback::drop_sink(ptr as *mut c_void);
            }
            if self.owns_handle {
                sys::MaaControllerDestroy(self.handle.as_ptr());
            }
        }
    }
}

/// Builder for ADB controller configuration.
///
/// Provides a fluent API for configuring ADB controllers with sensible defaults.
#[cfg(feature = "adb")]
pub struct AdbControllerBuilder {
    adb_path: String,
    address: String,
    screencap_methods: sys::MaaAdbScreencapMethod,
    input_methods: sys::MaaAdbInputMethod,
    config: String,
    agent_path: String,
}

#[cfg(feature = "adb")]
impl AdbControllerBuilder {
    /// Create a new builder with required ADB path and device address.
    pub fn new(adb_path: &str, address: &str) -> Self {
        Self {
            adb_path: adb_path.to_string(),
            address: address.to_string(),
            screencap_methods: sys::MaaAdbScreencapMethod_Default as sys::MaaAdbScreencapMethod,
            input_methods: sys::MaaAdbInputMethod_Default as sys::MaaAdbInputMethod,
            config: "{}".to_string(),
            agent_path: String::new(),
        }
    }

    /// Set the screencap methods to use.
    pub fn screencap_methods(mut self, methods: sys::MaaAdbScreencapMethod) -> Self {
        self.screencap_methods = methods;
        self
    }

    /// Set the input methods to use.
    pub fn input_methods(mut self, methods: sys::MaaAdbInputMethod) -> Self {
        self.input_methods = methods;
        self
    }

    /// Set additional configuration as JSON.
    pub fn config(mut self, config: &str) -> Self {
        self.config = config.to_string();
        self
    }

    /// Set the path to MaaAgentBinary.
    pub fn agent_path(mut self, path: &str) -> Self {
        self.agent_path = path.to_string();
        self
    }

    /// Build the controller with the configured options.
    pub fn build(self) -> MaaResult<Controller> {
        Controller::create_adb(
            &self.adb_path,
            &self.address,
            self.screencap_methods,
            self.input_methods,
            &self.config,
            &self.agent_path,
        )
    }
}

/// A borrowed reference to a Controller.
///
/// This is a non-owning view that can be used for read-only operations.
/// It does NOT call destroy when dropped and should only be used while
/// the underlying Controller is still alive.
pub struct ControllerRef<'a> {
    handle: *mut sys::MaaController,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> std::fmt::Debug for ControllerRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControllerRef")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<'a> ControllerRef<'a> {
    pub(crate) fn from_ptr(handle: *mut sys::MaaController) -> Option<Self> {
        if handle.is_null() {
            None
        } else {
            Some(Self {
                handle,
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// Check if connected.
    pub fn connected(&self) -> bool {
        unsafe { sys::MaaControllerConnected(self.handle) != 0 }
    }

    /// Get device UUID.
    pub fn uuid(&self) -> MaaResult<String> {
        let buffer = crate::buffer::MaaStringBuffer::new()?;
        let ret = unsafe { sys::MaaControllerGetUuid(self.handle, buffer.as_ptr()) };
        if ret != 0 {
            Ok(buffer.to_string())
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get device resolution.
    pub fn resolution(&self) -> MaaResult<(i32, i32)> {
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        let ret = unsafe { sys::MaaControllerGetResolution(self.handle, &mut width, &mut height) };
        if ret != 0 {
            Ok((width, height))
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get operation status.
    pub fn status(&self, ctrl_id: common::MaaId) -> common::MaaStatus {
        let s = unsafe { sys::MaaControllerStatus(self.handle, ctrl_id) };
        common::MaaStatus(s)
    }

    /// Wait for operation to complete.
    pub fn wait(&self, ctrl_id: common::MaaId) -> common::MaaStatus {
        let s = unsafe { sys::MaaControllerWait(self.handle, ctrl_id) };
        common::MaaStatus(s)
    }

    /// Get cached screenshot.
    pub fn cached_image(&self) -> MaaResult<crate::buffer::MaaImageBuffer> {
        let buffer = crate::buffer::MaaImageBuffer::new()?;
        let ret = unsafe { sys::MaaControllerCachedImage(self.handle, buffer.as_ptr()) };
        if ret != 0 {
            Ok(buffer)
        } else {
            Err(MaaError::FrameworkError(0))
        }
    }

    /// Get raw handle.
    pub fn raw(&self) -> *mut sys::MaaController {
        self.handle
    }
}
