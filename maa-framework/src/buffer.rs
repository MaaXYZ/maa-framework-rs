//! Safe buffer types for FFI data exchange.
//!
//! These buffers provide RAII-based memory management for data passed between
//! Rust and the C API. All buffers are automatically deallocated when dropped.
//!
//! # Buffer Types
//!
//! - [`MaaStringBuffer`] - UTF-8 string data
//! - [`MaaImageBuffer`] - Image data (supports PNG/JPEG encoding)
//! - [`MaaStringListBuffer`] - List of strings
//! - [`MaaImageListBuffer`] - List of images
//! - [`MaaRectBuffer`] - Rectangle coordinates

use std::ffi::CString;

use std::ptr::NonNull;
use std::slice;
use std::str;

use crate::{sys, MaaError, MaaResult};

/// Macro to implement common lifecycle methods for buffers.
macro_rules! impl_buffer_lifecycle {
    ($name:ident, $sys_type:ty, $create_fn:path, $destroy_fn:path) => {
        unsafe impl Send for $name {}

        impl $name {
            /// Create a new buffer.
            pub fn new() -> MaaResult<Self> {
                let handle = unsafe { $create_fn() };
                NonNull::new(handle)
                    .map(|ptr| Self {
                        handle: ptr,
                        own: true,
                    })
                    .ok_or(MaaError::NullPointer)
            }

            /// Create from an existing handle.
            ///
            /// # Safety
            ///
            /// This function assumes the handle is valid. The returned buffer will
            /// NOT take ownership of the handle (it won't be destroyed on drop).
            /// Use this when you are borrowing a handle from the C API.
            pub unsafe fn from_raw(handle: *mut $sys_type) -> Self {
                Self {
                    handle: unsafe { NonNull::new_unchecked(handle) },
                    own: false,
                }
            }

            /// Create from an existing handle safely (checks for null).
            /// Returns `None` if handle is null.
            pub fn from_handle(handle: *mut $sys_type) -> Option<Self> {
                NonNull::new(handle).map(|ptr| Self {
                    handle: ptr,
                    own: false,
                })
            }

            /// Get the underlying raw handle.
            #[inline]
            pub fn as_ptr(&self) -> *mut $sys_type {
                self.handle.as_ptr()
            }

            /// Get the underlying raw handle (alias for `as_ptr`).
            #[inline]
            pub fn raw(&self) -> *mut $sys_type {
                self.handle.as_ptr()
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if self.own {
                    unsafe { $destroy_fn(self.handle.as_ptr()) }
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new().expect(concat!("Failed to create ", stringify!($name)))
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("handle", &self.handle)
                    .field("own", &self.own)
                    .finish()
            }
        }
    };
}

/// A string buffer for UTF-8 text data.
///
/// Used for passing strings between Rust and the C API.
/// Automatically freed when dropped.
pub struct MaaStringBuffer {
    handle: NonNull<sys::MaaStringBuffer>,
    own: bool,
}

impl_buffer_lifecycle!(
    MaaStringBuffer,
    sys::MaaStringBuffer,
    sys::MaaStringBufferCreate,
    sys::MaaStringBufferDestroy
);

impl MaaStringBuffer {
    /// Set the buffer content from a string.
    pub fn set<S: AsRef<str>>(&mut self, content: S) -> MaaResult<()> {
        let s = content.as_ref();
        let c_str = CString::new(s)?;
        let ret = unsafe { sys::MaaStringBufferSet(self.handle.as_ptr(), c_str.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Set the buffer content from raw bytes (zero-copy if possible).
    ///
    /// This uses `MaaStringBufferSetEx` which accepts a length, allowing
    /// passing bytes without explicit null termination if the API supports it.
    pub fn set_ex<B: AsRef<[u8]>>(&mut self, content: B) -> MaaResult<()> {
        let bytes = content.as_ref();
        let ret = unsafe {
            sys::MaaStringBufferSetEx(
                self.handle.as_ptr(),
                bytes.as_ptr() as *const _,
                bytes.len() as u64,
            )
        };
        crate::common::check_bool(ret)
    }

    /// Get the buffer content as a string slice.
    pub fn as_str(&self) -> &str {
        let bytes = self.as_bytes();
        if bytes.is_empty() {
            ""
        } else {
            str::from_utf8(bytes).unwrap_or("")
        }
    }

    /// Get the buffer content as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let ptr = sys::MaaStringBufferGet(self.handle.as_ptr());
            let size = sys::MaaStringBufferSize(self.handle.as_ptr()) as usize;

            if ptr.is_null() || size == 0 {
                &[]
            } else {
                slice::from_raw_parts(ptr as *const u8, size)
            }
        }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { sys::MaaStringBufferIsEmpty(self.handle.as_ptr()) != 0 }
    }

    /// Clear the buffer.
    pub fn clear(&mut self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaStringBufferClear(self.handle.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Get the size of the buffer content (excluding null terminator).
    pub fn len(&self) -> usize {
        unsafe { sys::MaaStringBufferSize(self.handle.as_ptr()) as usize }
    }
}

impl std::fmt::Display for MaaStringBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for MaaStringBuffer {
    fn from(s: &str) -> Self {
        let mut buf = Self::new().expect("Failed to create MaaStringBuffer");
        buf.set(s).expect("Failed to set string buffer content");
        buf
    }
}

impl AsRef<str> for MaaStringBuffer {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Image buffer for storing and manipulating image data.
///
/// Supports raw BGR data and encoded formats (PNG/JPEG).
/// Compatible with OpenCV image types.
pub struct MaaImageBuffer {
    handle: NonNull<sys::MaaImageBuffer>,
    own: bool,
}

impl_buffer_lifecycle!(
    MaaImageBuffer,
    sys::MaaImageBuffer,
    sys::MaaImageBufferCreate,
    sys::MaaImageBufferDestroy
);

impl MaaImageBuffer {
    /// Check if the buffer is empty (contains no image data).
    pub fn is_empty(&self) -> bool {
        unsafe { sys::MaaImageBufferIsEmpty(self.handle.as_ptr()) != 0 }
    }

    /// Clear the buffer, releasing any image data.
    pub fn clear(&mut self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaImageBufferClear(self.handle.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Get the image width in pixels.
    pub fn width(&self) -> i32 {
        unsafe { sys::MaaImageBufferWidth(self.handle.as_ptr()) }
    }

    /// Get the image height in pixels.
    pub fn height(&self) -> i32 {
        unsafe { sys::MaaImageBufferHeight(self.handle.as_ptr()) }
    }

    /// Get the number of color channels (e.g., 3 for RGB, 4 for RGBA).
    pub fn channels(&self) -> i32 {
        unsafe { sys::MaaImageBufferChannels(self.handle.as_ptr()) }
    }

    /// Get the OpenCV image type constant.
    pub fn image_type(&self) -> i32 {
        unsafe { sys::MaaImageBufferType(self.handle.as_ptr()) }
    }

    /// Get the encoded image data as a byte vector (PNG format).
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        unsafe {
            let ptr = sys::MaaImageBufferGetEncoded(self.handle.as_ptr());
            let size = sys::MaaImageBufferGetEncodedSize(self.handle.as_ptr());
            if !ptr.is_null() && size > 0 {
                let slice = slice::from_raw_parts(ptr, size as usize);
                Some(slice.to_vec())
            } else {
                None
            }
        }
    }

    /// Get the raw image data (BGR format, row-major).
    pub fn raw_data(&self) -> Option<&[u8]> {
        unsafe {
            let ptr = sys::MaaImageBufferGetRawData(self.handle.as_ptr());
            if ptr.is_null() {
                return None;
            }
            let w = self.width() as usize;
            let h = self.height() as usize;
            let c = self.channels() as usize;
            if w == 0 || h == 0 || c == 0 {
                return None;
            }
            Some(slice::from_raw_parts(ptr as *const u8, w * h * c))
        }
    }

    /// Set the image from raw BGR data.
    pub fn set_raw_data(
        &mut self,
        data: &[u8],
        width: i32,
        height: i32,
        img_type: i32,
    ) -> MaaResult<()> {
        let ret = unsafe {
            sys::MaaImageBufferSetRawData(
                self.handle.as_ptr(),
                data.as_ptr() as *mut std::ffi::c_void,
                width,
                height,
                img_type,
            )
        };
        crate::common::check_bool(ret)
    }

    /// Set the image from raw BGR data (assuming 3 channels, CV_8UC3).
    pub fn set(&mut self, data: &[u8], width: i32, height: i32) -> MaaResult<()> {
        self.set_raw_data(data, width, height, 16)
    }

    /// Set the image from encoded data (PNG/JPEG).
    pub fn set_encoded(&mut self, data: &[u8]) -> MaaResult<()> {
        let ret = unsafe {
            sys::MaaImageBufferSetEncoded(
                self.handle.as_ptr(),
                data.as_ptr() as *mut u8,
                data.len() as u64,
            )
        };
        crate::common::check_bool(ret)
    }

    /// Resize the image.
    ///
    /// * `width`: Target width, 0 for auto aspect ratio.
    /// * `height`: Target height, 0 for auto aspect ratio.
    pub fn resize(&mut self, width: i32, height: i32) -> MaaResult<()> {
        let ret = unsafe { sys::MaaImageBufferResize(self.handle.as_ptr(), width, height) };
        crate::common::check_bool(ret)
    }

    /// Convert to `image` crate's `DynamicImage`.
    #[cfg(feature = "image")]
    pub fn to_dynamic_image(&self) -> MaaResult<image::DynamicImage> {
        let encoded = self.to_vec().ok_or(MaaError::ImageConversionError)?;
        image::load_from_memory(&encoded).map_err(|_| MaaError::ImageConversionError)
    }

    /// Create an image buffer from `image` crate's `DynamicImage`.
    #[cfg(feature = "image")]
    pub fn from_dynamic_image(img: &image::DynamicImage) -> MaaResult<Self> {
        use std::io::Cursor;

        let mut bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .map_err(|_| MaaError::ImageConversionError)?;

        let mut buffer = Self::new()?;
        buffer.set_encoded(&bytes)?;
        Ok(buffer)
    }

    /// Create an image buffer from an RGB image.
    #[cfg(feature = "image")]
    pub fn from_rgb_image(img: &image::RgbImage) -> MaaResult<Self> {
        Self::from_dynamic_image(&image::DynamicImage::ImageRgb8(img.clone()))
    }

    /// Create an image buffer from an RGBA image.
    #[cfg(feature = "image")]
    pub fn from_rgba_image(img: &image::RgbaImage) -> MaaResult<Self> {
        Self::from_dynamic_image(&image::DynamicImage::ImageRgba8(img.clone()))
    }
}

/// A list buffer for storing multiple images.
///
/// Provides indexed access and iteration over a collection of [`MaaImageBuffer`]s.
pub struct MaaImageListBuffer {
    handle: NonNull<sys::MaaImageListBuffer>,
    own: bool,
}

impl_buffer_lifecycle!(
    MaaImageListBuffer,
    sys::MaaImageListBuffer,
    sys::MaaImageListBufferCreate,
    sys::MaaImageListBufferDestroy
);

impl MaaImageListBuffer {
    /// Returns the number of images in the list.
    pub fn len(&self) -> usize {
        unsafe { sys::MaaImageListBufferSize(self.handle.as_ptr()) as usize }
    }

    /// Returns `true` if the list contains no images.
    pub fn is_empty(&self) -> bool {
        unsafe { sys::MaaImageListBufferIsEmpty(self.handle.as_ptr()) != 0 }
    }

    /// Get image at index. Returns None if index out of bounds.
    /// Note: The returned buffer is a view into this list (non-owning).
    pub fn at(&self, index: usize) -> Option<MaaImageBuffer> {
        unsafe {
            let ptr = sys::MaaImageListBufferAt(self.handle.as_ptr(), index as u64);
            MaaImageBuffer::from_handle(ptr as *mut sys::MaaImageBuffer)
        }
    }

    /// Appends an image to the end of the list.
    pub fn append(&self, image: &MaaImageBuffer) -> MaaResult<()> {
        let ret = unsafe { sys::MaaImageListBufferAppend(self.handle.as_ptr(), image.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Set the content of this list, replacing existing content.
    pub fn set(&self, data: &[&MaaImageBuffer]) -> MaaResult<()> {
        self.clear()?;
        for img in data {
            self.append(img)?;
        }
        Ok(())
    }

    /// Removes the image at the specified index.
    pub fn remove(&self, index: usize) -> MaaResult<()> {
        let ret = unsafe { sys::MaaImageListBufferRemove(self.handle.as_ptr(), index as u64) };
        crate::common::check_bool(ret)
    }

    /// Removes all images from the list.
    pub fn clear(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaImageListBufferClear(self.handle.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Get all images as a vector of `MaaImageBuffer` handles.
    pub fn to_vec(&self) -> Vec<MaaImageBuffer> {
        (0..self.len()).filter_map(|i| self.at(i)).collect()
    }

    /// Get an iterator over the images in the list.
    pub fn iter(&self) -> impl Iterator<Item = MaaImageBuffer> + '_ {
        (0..self.len()).filter_map(move |i| self.at(i))
    }

    /// Get all images as a vector of encoded byte vectors.
    pub fn to_vec_of_vec(&self) -> Vec<Vec<u8>> {
        self.iter().filter_map(|img| img.to_vec()).collect()
    }

    /// Get all images as raw BGR data vectors + metadata.
    pub fn to_raw_vecs(&self) -> Vec<(Vec<u8>, i32, i32, i32, i32)> {
        self.iter()
            .filter_map(|img| {
                img.raw_data().map(|data| {
                    (
                        data.to_vec(),
                        img.width(),
                        img.height(),
                        img.channels(),
                        img.image_type(),
                    )
                })
            })
            .collect()
    }
}

/// A list buffer for storing multiple strings.
///
/// Provides indexed access and iteration over a collection of UTF-8 strings.
pub struct MaaStringListBuffer {
    handle: NonNull<sys::MaaStringListBuffer>,
    own: bool,
}

impl_buffer_lifecycle!(
    MaaStringListBuffer,
    sys::MaaStringListBuffer,
    sys::MaaStringListBufferCreate,
    sys::MaaStringListBufferDestroy
);

impl MaaStringListBuffer {
    /// Returns the number of strings in the list.
    pub fn len(&self) -> usize {
        unsafe { sys::MaaStringListBufferSize(self.handle.as_ptr()) as usize }
    }

    /// Returns `true` if the list contains no strings.
    pub fn is_empty(&self) -> bool {
        unsafe { sys::MaaStringListBufferIsEmpty(self.handle.as_ptr()) != 0 }
    }

    /// Appends a string to the end of the list.
    pub fn append(&self, s: &str) -> MaaResult<()> {
        let mut str_buf = MaaStringBuffer::new()?;
        str_buf.set(s)?;
        let ret = unsafe { sys::MaaStringListBufferAppend(self.handle.as_ptr(), str_buf.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Set the content of this list, replacing existing content.
    pub fn set<S: AsRef<str>>(&self, data: &[S]) -> MaaResult<()> {
        self.clear()?;
        for s in data {
            self.append(s.as_ref())?;
        }
        Ok(())
    }

    /// Removes the string at the specified index.
    pub fn remove(&self, index: usize) -> MaaResult<()> {
        let ret = unsafe { sys::MaaStringListBufferRemove(self.handle.as_ptr(), index as u64) };
        crate::common::check_bool(ret)
    }

    /// Removes all strings from the list.
    pub fn clear(&self) -> MaaResult<()> {
        let ret = unsafe { sys::MaaStringListBufferClear(self.handle.as_ptr()) };
        crate::common::check_bool(ret)
    }

    /// Get an iterator over the strings in the list.
    ///
    /// The iterator yields `&str` slices borrowing from the buffer.
    /// This is zero-cost and safe.
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        (0..self.len()).map(move |i| unsafe {
            let ptr = sys::MaaStringListBufferAt(self.handle.as_ptr(), i as u64);
            // ptr is MaaStringBufferHandle (not null usually if index is valid)
            if ptr.is_null() {
                return "";
            }
            let str_ptr = sys::MaaStringBufferGet(ptr as *mut sys::MaaStringBuffer);
            let size = sys::MaaStringBufferSize(ptr as *mut sys::MaaStringBuffer) as usize;
            if str_ptr.is_null() || size == 0 {
                ""
            } else {
                let slice = slice::from_raw_parts(str_ptr as *const u8, size);
                str::from_utf8(slice).unwrap_or("")
            }
        })
    }

    /// Collects all strings into a `Vec<String>`.
    pub fn to_vec(&self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

/// Rect buffer for passing rectangle coordinates between Rust and C API.
///
/// Stores x, y position and width, height dimensions.
pub struct MaaRectBuffer {
    handle: NonNull<sys::MaaRect>,
    own: bool,
}

impl_buffer_lifecycle!(
    MaaRectBuffer,
    sys::MaaRect,
    sys::MaaRectCreate,
    sys::MaaRectDestroy
);

impl MaaRectBuffer {
    /// Get the rect values.
    pub fn get(&self) -> crate::common::Rect {
        unsafe {
            crate::common::Rect {
                x: sys::MaaRectGetX(self.handle.as_ptr()),
                y: sys::MaaRectGetY(self.handle.as_ptr()),
                width: sys::MaaRectGetW(self.handle.as_ptr()),
                height: sys::MaaRectGetH(self.handle.as_ptr()),
            }
        }
    }

    /// Set the rect values.
    pub fn set(&mut self, rect: &crate::common::Rect) -> MaaResult<()> {
        let ret = unsafe {
            sys::MaaRectSet(
                self.handle.as_ptr(),
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            )
        };
        crate::common::check_bool(ret)
    }
}

impl From<crate::common::Rect> for MaaRectBuffer {
    fn from(rect: crate::common::Rect) -> Self {
        let mut buf = Self::new().expect("Failed to create MaaRectBuffer");
        buf.set(&rect).expect("Failed to set rect buffer");
        buf
    }
}

impl From<MaaRectBuffer> for crate::common::Rect {
    fn from(buf: MaaRectBuffer) -> Self {
        buf.get()
    }
}

impl From<&MaaRectBuffer> for crate::common::Rect {
    fn from(buf: &MaaRectBuffer) -> Self {
        buf.get()
    }
}
