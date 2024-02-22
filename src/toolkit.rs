use serde::{Deserialize, Serialize};

use crate::{
    error::Error, instance::MaaInstance, internal, maa_bool, string, string_view, MaaResult,
};

#[cfg(feature = "win32")]
use crate::controller::win32::MaaWin32Hwnd;

#[cfg(feature = "adb")]
use crate::controller::adb::MaaAdbControllerType;

pub struct MaaToolkit;

impl MaaToolkit {
    /// Initialize the MaaToolkit
    ///
    /// # Errors
    ///
    /// Returns an error if the toolkit initialization fails
    pub fn new() -> MaaResult<Self> {
        let toolkit_init_ret = unsafe { internal::MaaToolkitInit() };

        if !maa_bool!(toolkit_init_ret) {
            return Err(Error::MaaToolkitInitError);
        }

        Ok(Self)
    }

    /// Find all the devices
    ///
    /// # Errors
    ///
    /// Return an error if fails to convert MaaStringView to String
    #[cfg(feature = "adb")]
    pub fn find_adb_device() -> MaaResult<Vec<AdbDeviceInfo>> {
        let device_count = unsafe { internal::MaaToolkitPostFindDevice() };

        Self::get_adb_devices_info(device_count.into())
    }

    /// Find all the devices with a given adb path
    ///
    /// # Errors
    ///
    /// Return an error if fails to convert MaaStringView to String
    #[cfg(feature = "adb")]
    pub fn find_adb_device_with_adb(adb_path: &str) -> MaaResult<Vec<AdbDeviceInfo>> {
        let adb_path = string_view!(adb_path);
        let device_count = unsafe { internal::MaaToolkitPostFindDeviceWithAdb(adb_path) };

        Self::get_adb_devices_info(device_count.into())
    }

    #[cfg(feature = "adb")]
    fn get_adb_devices_info(device_count: u64) -> MaaResult<Vec<AdbDeviceInfo>> {
        let mut devices = Vec::with_capacity(device_count as usize);

        for i in 0..device_count {
            let name = unsafe { internal::MaaToolkitGetDeviceName(i) };
            let adb_path = unsafe { internal::MaaToolkitGetDeviceAdbPath(i) };
            let adb_serial = unsafe { internal::MaaToolkitGetDeviceAdbSerial(i) };
            let adb_controller_type = unsafe { internal::MaaToolkitGetDeviceAdbControllerType(i) };
            let adb_config = unsafe { internal::MaaToolkitGetDeviceAdbConfig(i) };

            let name = string!(name);
            let adb_path = string!(adb_path);
            let adb_serial = string!(adb_serial);
            let adb_config = string!(adb_config);
            let adb_controller_type = MaaAdbControllerType::try_from(adb_controller_type)?;

            devices.push(AdbDeviceInfo {
                name,
                adb_path,
                adb_serial,
                adb_controller_type,
                adb_config,
            });
        }

        Ok(devices)
    }

    pub fn register_custom_recognizer_executor<T>(
        handle: MaaInstance<T>,
        recognizer_name: &str,
        recognizer_exec_path: &str,
        recognizer_exec_param_json: &str,
    ) -> MaaResult<()> {
        let recognizer_name = string_view!(recognizer_name);
        let recognizer_exec_path = string_view!(recognizer_exec_path);
        let recognizer_exec_param_json = string_view!(recognizer_exec_param_json);

        let ret = unsafe {
            internal::MaaToolkitRegisterCustomRecognizerExecutor(
                *handle,
                recognizer_name,
                recognizer_exec_path,
                recognizer_exec_param_json,
            )
        };

        if !maa_bool!(ret) {
            return Err(Error::MaaToolkitRegisterCustomRecognizerExecutorError);
        }

        Ok(())
    }

    pub fn unregister_custom_recognizer_executor<T>(
        handle: MaaInstance<T>,
        recognizer_name: &str,
    ) -> MaaResult<()> {
        let recognizer_name = string_view!(recognizer_name);

        let ret = unsafe {
            internal::MaaToolkitUnregisterCustomRecognizerExecutor(*handle, recognizer_name)
        };

        if !maa_bool!(ret) {
            return Err(Error::MaaToolkitUnregisterCustomRecognizerExecutorError);
        }

        Ok(())
    }

    pub fn register_custom_action_executor<T>(
        handle: MaaInstance<T>,
        action_name: &str,
        action_exec_path: &str,
        action_exec_param_json: &str,
    ) -> MaaResult<()> {
        let action_name = string_view!(action_name);
        let action_exec_path = string_view!(action_exec_path);
        let action_exec_param_json = string_view!(action_exec_param_json);

        let ret = unsafe {
            internal::MaaToolkitRegisterCustomActionExecutor(
                *handle,
                action_name,
                action_exec_path,
                action_exec_param_json,
            )
        };

        if !maa_bool!(ret) {
            return Err(Error::MaaToolkitRegisterCustomRecognizerExecutorError);
        }

        Ok(())
    }

    pub fn unregister_custom_action_executor<T>(
        handle: MaaInstance<T>,
        action_name: &str,
    ) -> MaaResult<()> {
        let action_name = string_view!(action_name);

        let ret =
            unsafe { internal::MaaToolkitUnregisterCustomActionExecutor(*handle, action_name) };

        if !maa_bool!(ret) {
            return Err(Error::MaaToolkitUnregisterCustomRecognizerExecutorError);
        }

        Ok(())
    }

    /// Find all the windows with a given class name and window name
    ///
    /// # Parameters
    /// - `class_name`: The class name of the window
    /// - `window_name`: The window name of the window
    /// - `find`: If true, find the window using system win32 api, otherwise search the window with text match
    #[cfg(feature = "win32")]
    pub fn find_win32_window(class_name: &str, window_name: &str, find: bool) -> Vec<MaaWin32Hwnd> {
        let class_name = string_view!(class_name);
        let window_name = string_view!(window_name);

        let hwnd_count = unsafe {
            if find {
                internal::MaaToolkitFindWindow(class_name, window_name)
            } else {
                internal::MaaToolkitSearchWindow(class_name, window_name)
            }
        };

        let mut hwnds = Vec::with_capacity(hwnd_count as usize);

        for i in 0..hwnd_count {
            let hwnd = unsafe { internal::MaaToolkitGetWindow(i) };
            hwnds.push(MaaWin32Hwnd(hwnd));
        }

        hwnds
    }

    #[cfg(feature = "win32")]
    pub fn get_cursor_window() -> MaaWin32Hwnd {
        let hwnd = unsafe { internal::MaaToolkitGetCursorWindow() };
        MaaWin32Hwnd(hwnd)
    }

    #[cfg(feature = "win32")]
    pub fn get_desktop_window() -> MaaWin32Hwnd {
        let hwnd = unsafe { internal::MaaToolkitGetDesktopWindow() };
        MaaWin32Hwnd(hwnd)
    }

    #[cfg(feature = "win32")]
    pub fn get_foreground_window() -> MaaWin32Hwnd {
        let hwnd = unsafe { internal::MaaToolkitGetForegroundWindow() };
        MaaWin32Hwnd(hwnd)
    }
}

impl Drop for MaaToolkit {
    fn drop(&mut self) {
        unsafe { internal::MaaToolkitUninit() };
    }
}

unsafe impl Send for MaaToolkit {}
unsafe impl Sync for MaaToolkit {}

#[derive(Serialize, Deserialize)]
#[cfg(feature = "adb")]
pub struct AdbDeviceInfo {
    pub name: String,
    pub adb_path: String,
    pub adb_serial: String,
    pub adb_controller_type: MaaAdbControllerType,
    pub adb_config: String,
}

#[cfg(test)]
mod test {
    use super::MaaToolkit;

    #[test]
    fn test_init() {
        let toolkit = MaaToolkit::new();

        assert!(toolkit.is_ok());
    }
}
