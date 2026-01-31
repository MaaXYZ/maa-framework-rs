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
                let lib = libloading::os::windows::Library::load_with_flags(&p, 0x00000008)?;
                Ok(lib.into())
            }
            #[cfg(not(target_os = "windows"))]
            {
                let p = p.canonicalize().unwrap_or(p.to_path_buf());
                libloading::Library::new(&p)
            }
        };

        libs.push(load_lib(path)?);

        if let Some(parent) = path.parent() {
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            let (prefix, suffix) = if file_name.ends_with(".dll") {
                ("", ".dll")
            } else if file_name.starts_with("lib") && file_name.ends_with(".dylib") {
                ("lib", ".dylib")
            } else if file_name.starts_with("lib") && file_name.ends_with(".so") {
                ("lib", ".so")
            } else {
                ("", "")
            };

            let mut try_load = |name: &str| {
                let p = parent.join(format!("{}{}{}", prefix, name, suffix));
                if p.exists() {
                    if let Ok(lib) = load_lib(&p) {
                        libs.push(lib);
                    }
                }
            };

            try_load("MaaToolkit");
            try_load("MaaAgentServer");
            try_load("MaaAgentClient");
        }

        Ok(Self { libs })
    }

    pub unsafe fn get<T>(
        &self,
        symbol: &[u8],
    ) -> Result<libloading::Symbol<'_, T>, libloading::Error> {
        for lib in &self.libs {
            if let Ok(s) = lib.get(symbol) {
                return Ok(s);
            }
        }
        self.libs[0].get(symbol)
    }
}

#[cfg(feature = "dynamic")]
pub unsafe fn load_library(path: &std::path::Path) -> Result<(), String> {
    if INSTANCE.get().is_some() {
        return Err("Library already loaded".to_string());
    }
    let lib = MaaFramework::new(path).map_err(|e| e.to_string())?;
    INSTANCE
        .set(lib)
        .map_err(|_| "Library already loaded".to_string())
}

#[cfg(feature = "dynamic")]
macro_rules! shim {
    ($name:ident ( $($arg:ident : $type:ty),* $(,)? ) -> $ret:ty) => {
        pub unsafe fn $name($($arg : $type),*) -> $ret {
            let lib = INSTANCE.get().expect("MaaFramework library not loaded!");
            (lib.$name)($($arg),*)
        }
    }
}

#[cfg(feature = "dynamic")]
include!(concat!(env!("OUT_DIR"), "/shims.rs"));
