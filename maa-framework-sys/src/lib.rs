//! Raw FFI bindings to the C API (auto-generated).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(rustdoc::broken_intra_doc_links)]

#[cfg(not(feature = "dynamic"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "dynamic")]
mod dynamic {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(rustdoc::broken_intra_doc_links)]
    use super::CompositeLibrary;
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(feature = "dynamic")]
pub use dynamic::*;

#[cfg(feature = "dynamic")]
static INSTANCE: once_cell::sync::OnceCell<MaaFramework> = once_cell::sync::OnceCell::new();

#[cfg(feature = "dynamic")]
pub struct CompositeLibrary {
    libs: Vec<libloading::Library>,
}

#[cfg(feature = "dynamic")]
impl CompositeLibrary {
    pub unsafe fn new<P: AsRef<std::ffi::OsStr>>(path: P) -> Result<Self, libloading::Error> {
        let path = std::path::Path::new(path.as_ref());
        let mut libs = Vec::new();

        let load_lib = |p: &std::path::Path| -> Result<libloading::Library, libloading::Error> {
            #[cfg(target_os = "windows")]
            {
                //LOAD_WITH_ALTERED_SEARCH_PATH (0x00000008)
                let p = p.canonicalize().unwrap_or(p.to_path_buf());
                let lib =
                    unsafe { libloading::os::windows::Library::load_with_flags(&p, 0x00000008)? };
                Ok(lib.into())
            }
            #[cfg(not(target_os = "windows"))]
            {
                let p = p.canonicalize().unwrap_or(p.to_path_buf());
                unsafe { libloading::Library::new(&p) }
            }
        };

        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let dir = path.parent().unwrap_or(std::path::Path::new(""));

        let prefix = if cfg!(target_os = "windows") {
            ""
        } else {
            "lib"
        };
        let ext = if cfg!(target_os = "windows") {
            ".dll"
        } else if cfg!(target_os = "macos") {
            ".dylib"
        } else {
            ".so"
        };

        let try_load = |name: &str| {
            let p = dir.join(format!("{}{}{}", prefix, name, ext));
            load_lib(&p).ok()
        };

        let is_agent_server = file_name.contains("MaaAgentServer");

        if !is_agent_server {
            if let Some(lib) = try_load("MaaToolkit") {
                libs.push(lib);
            }
        }

        let main_lib = load_lib(path)?;
        libs.insert(0, main_lib);

        if !is_agent_server {
            if let Some(lib) = try_load("MaaAgentClient") {
                libs.push(lib);
            }
        }

        Ok(Self { libs })
    }

    pub unsafe fn get<T>(
        &self,
        symbol: &[u8],
    ) -> Result<libloading::Symbol<'_, T>, libloading::Error> {
        for lib in &self.libs {
            if let Ok(s) = unsafe { lib.get(symbol) } {
                return Ok(s);
            }
        }
        unsafe { self.libs[0].get(symbol) }
    }
}

#[cfg(feature = "dynamic")]
pub unsafe fn load_library(path: &std::path::Path) -> Result<(), String> {
    if INSTANCE.get().is_some() {
        return Err("Library already loaded".to_string());
    }
    let lib = unsafe { MaaFramework::new(path).map_err(|e| e.to_string())? };
    INSTANCE
        .set(lib)
        .map_err(|_| "Library already loaded".to_string())
}

/// Check all symbols required to create and safely destroy an iOS controller.
///
/// Missing optional symbols do not prevent loading an older framework. Call this
/// before creating an owning controller to avoid a missing-symbol panic in Drop.
#[cfg(feature = "dynamic")]
pub fn require_ios_controller_api() -> Result<(), String> {
    let lib = INSTANCE
        .get()
        .ok_or_else(|| "<library not loaded>".to_string())?;
    for (name, present) in [
        ("MaaIOSControllerCreate", lib.MaaIOSControllerCreate.is_ok()),
        ("MaaControllerDestroy", lib.MaaControllerDestroy.is_ok()),
        (
            "MaaControllerClearSinks",
            lib.MaaControllerClearSinks.is_ok(),
        ),
    ] {
        if !present {
            return Err(name.to_string());
        }
    }
    Ok(())
}

/// Check the complete iOS discovery symbol set before allocating a list.
///
/// This includes destruction and every accessor, so an older or partial toolkit
/// cannot cause a missing-symbol panic while a list guard is being unwound.
#[cfg(feature = "dynamic")]
pub fn require_ios_discovery_api(with_options: bool) -> Result<(), String> {
    let lib = INSTANCE
        .get()
        .ok_or_else(|| "<library not loaded>".to_string())?;
    for (name, present) in [
        (
            "MaaToolkitIOSDeviceListCreate",
            lib.MaaToolkitIOSDeviceListCreate.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceListDestroy",
            lib.MaaToolkitIOSDeviceListDestroy.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceListSize",
            lib.MaaToolkitIOSDeviceListSize.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceListAt",
            lib.MaaToolkitIOSDeviceListAt.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceGetName",
            lib.MaaToolkitIOSDeviceGetName.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceGetUdid",
            lib.MaaToolkitIOSDeviceGetUdid.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceGetState",
            lib.MaaToolkitIOSDeviceGetState.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceGetConnections",
            lib.MaaToolkitIOSDeviceGetConnections.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceGetScreencapMethods",
            lib.MaaToolkitIOSDeviceGetScreencapMethods.is_ok(),
        ),
        (
            "MaaToolkitIOSDeviceListGetWarnings",
            lib.MaaToolkitIOSDeviceListGetWarnings.is_ok(),
        ),
    ] {
        if !present {
            return Err(name.to_string());
        }
    }
    let (name, present) = if with_options {
        (
            "MaaToolkitIOSDeviceFindWithOptions",
            lib.MaaToolkitIOSDeviceFindWithOptions.is_ok(),
        )
    } else {
        (
            "MaaToolkitIOSDeviceFind",
            lib.MaaToolkitIOSDeviceFind.is_ok(),
        )
    };
    if !present {
        return Err(name.to_string());
    }
    Ok(())
}

#[cfg(feature = "dynamic")]
macro_rules! shim {
    ($name:ident ( $($arg:ident : $type:ty),* $(,)? ) -> $ret:ty) => {
        pub unsafe fn $name($($arg : $type),*) -> $ret {
            let lib = INSTANCE.get().expect("MaaFramework library not loaded!");
            let func = match &lib.$name {
                Ok(f) => f,
                Err(e) => panic!("Function {} not loaded: {}", stringify!($name), e),
            };

            unsafe { func($($arg),*) }
        }
    }
}

#[cfg(feature = "dynamic")]
include!(concat!(env!("OUT_DIR"), "/shims.rs"));
