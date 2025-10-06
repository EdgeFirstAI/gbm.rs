//! # Safe `libgbm` bindings for [rust](https://www.rust-lang.org)
//!
//! The Generic Buffer Manager
//!
//! This module provides an abstraction that the caller can use to request a
//! buffer from the underlying memory management system for the platform.
//!
//! This allows the creation of portable code whilst still allowing access to
//! the underlying memory manager.
//!
//! This library is best used in combination with [`drm-rs`](https://github.com/Smithay/drm-rs),
//! provided through the `drm-support` feature.
//!
//! ## Example
//!
//! ```rust,no_run
//! # extern crate drm;
//! # extern crate gbm;
//! # use drm::control::connector::Info as ConnectorInfo;
//! # use drm::control::Mode;
//! use drm::control::{self, crtc, framebuffer};
//! use gbm::{BufferObjectFlags, Device, Format};
//!
//! # use std::fs::{File, OpenOptions};
//! # use std::os::unix::io::{AsFd, BorrowedFd};
//! #
//! # use drm::control::Device as ControlDevice;
//! # use drm::Device as BasicDevice;
//! # struct Card(File);
//! #
//! # impl AsFd for Card {
//! #     fn as_fd(&self) -> BorrowedFd {
//! #         self.0.as_fd()
//! #     }
//! # }
//! #
//! # impl BasicDevice for Card {}
//! # impl ControlDevice for Card {}
//! #
//! # fn init_drm_device() -> Card {
//! #     let mut options = OpenOptions::new();
//! #     options.read(true);
//! #     options.write(true);
//! #     let file = options.open("/dev/dri/card0").unwrap();
//! #     Card(file)
//! # }
//! # fn main() {
//! // ... init your drm device ...
//! let drm = init_drm_device();
//!
//! // init a GBM device
//! let gbm = Device::new(drm).unwrap();
//!
//! // create a 4x4 buffer
//! let mut bo = gbm
//!     .create_buffer_object::<()>(
//!         1280,
//!         720,
//!         Format::Argb8888,
//!         BufferObjectFlags::SCANOUT | BufferObjectFlags::WRITE,
//!     )
//!     .unwrap();
//! // write something to it (usually use import or egl rendering instead)
//! let buffer = {
//!     let mut buffer = Vec::new();
//!     for i in 0..1280 {
//!         for _ in 0..720 {
//!             buffer.push(if i % 2 == 0 { 0 } else { 255 });
//!         }
//!     }
//!     buffer
//! };
//! bo.write(&buffer).unwrap();
//!
//! // create a framebuffer from our buffer
//! let fb = gbm.add_framebuffer(&bo, 32, 32).unwrap();
//!
//! # let res_handles = gbm.resource_handles().unwrap();
//! # let con = *res_handles.connectors().iter().next().unwrap();
//! # let crtc_handle = *res_handles.crtcs().iter().next().unwrap();
//! # let connector_info: ConnectorInfo = gbm.get_connector(con, false).unwrap();
//! # let mode: Mode = connector_info.modes()[0];
//! #
//! // display it (and get a crtc, mode and connector before)
//! gbm.set_crtc(crtc_handle, Some(fb), (0, 0), &[con], Some(mode))
//!     .unwrap();
//! # }
//! ```
#![warn(missing_debug_implementations)]
#![deny(missing_docs)]

extern crate gbm_sys as ffi;

#[cfg(feature = "import-wayland")]
pub use wayland_server;

#[cfg(feature = "drm-support")]
pub use drm;

mod buffer_object;
mod device;
mod surface;

pub use self::{buffer_object::*, device::*, surface::*};
pub use drm_fourcc::{DrmFourcc as Format, DrmModifier as Modifier};

use std::{fmt, sync::Arc};

struct Gbm {
    #[cfg(feature = "dynamic")]
    lib: ffi::gbm,
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
impl Gbm {
    fn new() -> std::io::Result<Self> {
        #[cfg(not(feature = "dynamic"))]
        return Ok(Self {});
        #[cfg(feature = "dynamic")]
        match unsafe { ffi::gbm::new("libgbm.so") } {
            Err(e) => Err(std::io::Error::other(e)),
            Ok(lib) => Ok(Self { lib }),
        }
    }

    unsafe fn gbm_device_get_fd(&self, gbm: *mut ffi::gbm_device) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_device_get_fd(gbm);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_device_get_fd(gbm);
    }

    unsafe fn gbm_device_get_backend_name(&self, gbm: *mut ffi::gbm_device) -> *const libc::c_char {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_device_get_backend_name(gbm);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_device_get_backend_name(gbm);
    }

    unsafe fn gbm_device_is_format_supported(
        &self,
        gbm: *mut ffi::gbm_device,
        format: u32,
        flags: u32,
    ) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_device_is_format_supported(gbm, format, flags);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_device_is_format_supported(gbm, format, flags);
    }

    unsafe fn gbm_device_get_format_modifier_plane_count(
        &self,
        gbm: *mut ffi::gbm_device,
        format: u32,
        modifier: u64,
    ) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_device_get_format_modifier_plane_count(gbm, format, modifier);
        #[cfg(feature = "dynamic")]
        return self
            .lib
            .gbm_device_get_format_modifier_plane_count(gbm, format, modifier);
    }

    unsafe fn gbm_device_destroy(&self, gbm: *mut ffi::gbm_device) {
        #[cfg(not(feature = "dynamic"))]
        ffi::gbm_device_destroy(gbm);
        #[cfg(feature = "dynamic")]
        self.lib.gbm_device_destroy(gbm);
    }

    unsafe fn gbm_create_device(&self, fd: libc::c_int) -> *mut ffi::gbm_device {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_create_device(fd);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_create_device(fd);
    }

    unsafe fn gbm_bo_create(
        &self,
        gbm: *mut ffi::gbm_device,
        width: u32,
        height: u32,
        format: u32,
        flags: u32,
    ) -> *mut ffi::gbm_bo {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_create(gbm, width, height, format, flags);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_create(gbm, width, height, format, flags);
    }

    unsafe fn gbm_bo_create_with_modifiers(
        &self,
        gbm: *mut ffi::gbm_device,
        width: u32,
        height: u32,
        format: u32,
        modifiers: *const u64,
        count: libc::c_uint,
    ) -> *mut ffi::gbm_bo {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_create_with_modifiers(gbm, width, height, format, modifiers, count);
        #[cfg(feature = "dynamic")]
        return self
            .lib
            .gbm_bo_create_with_modifiers(gbm, width, height, format, modifiers, count);
    }

    unsafe fn gbm_bo_create_with_modifiers2(
        &self,
        gbm: *mut ffi::gbm_device,
        width: u32,
        height: u32,
        format: u32,
        modifiers: *const u64,
        count: libc::c_uint,
        flags: u32,
    ) -> *mut ffi::gbm_bo {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_create_with_modifiers2(
            gbm, width, height, format, modifiers, count, flags,
        );
        #[cfg(feature = "dynamic")]
        return self
            .lib
            .gbm_bo_create_with_modifiers2(gbm, width, height, format, modifiers, count, flags);
    }

    unsafe fn gbm_bo_import(
        &self,
        gbm: *mut ffi::gbm_device,
        type_: u32,
        buffer: *mut libc::c_void,
        flags: u32,
    ) -> *mut ffi::gbm_bo {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_import(gbm, type_, buffer, flags);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_import(gbm, type_, buffer, flags);
    }

    unsafe fn gbm_bo_map(
        &self,
        bo: *mut ffi::gbm_bo,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        flags: u32,
        stride: *mut u32,
        map_data: *mut *mut libc::c_void,
    ) -> *mut libc::c_void {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_map(bo, x, y, width, height, flags, stride, map_data);
        #[cfg(feature = "dynamic")]
        return self
            .lib
            .gbm_bo_map(bo, x, y, width, height, flags, stride, map_data);
    }

    unsafe fn gbm_bo_unmap(&self, bo: *mut ffi::gbm_bo, map_data: *mut libc::c_void) {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_unmap(bo, map_data);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_unmap(bo, map_data);
    }

    unsafe fn gbm_bo_get_width(&self, bo: *mut ffi::gbm_bo) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_width(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_width(bo);
    }

    unsafe fn gbm_bo_get_height(&self, bo: *mut ffi::gbm_bo) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_height(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_height(bo);
    }

    unsafe fn gbm_bo_get_stride(&self, bo: *mut ffi::gbm_bo) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_stride(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_stride(bo);
    }

    unsafe fn gbm_bo_get_stride_for_plane(&self, bo: *mut ffi::gbm_bo, plane: libc::c_int) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_stride_for_plane(bo, plane);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_stride_for_plane(bo, plane);
    }

    unsafe fn gbm_bo_get_format(&self, bo: *mut ffi::gbm_bo) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_format(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_format(bo);
    }

    unsafe fn gbm_bo_get_bpp(&self, bo: *mut ffi::gbm_bo) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_bpp(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_bpp(bo);
    }

    unsafe fn gbm_bo_get_offset(&self, bo: *mut ffi::gbm_bo, plane: libc::c_int) -> u32 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_offset(bo, plane);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_offset(bo, plane);
    }

    unsafe fn gbm_bo_get_device(&self, bo: *mut ffi::gbm_bo) -> *mut ffi::gbm_device {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_device(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_device(bo);
    }

    unsafe fn gbm_bo_get_handle(&self, bo: *mut ffi::gbm_bo) -> ffi::gbm_bo_handle {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_handle(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_handle(bo);
    }

    unsafe fn gbm_bo_get_fd(&self, bo: *mut ffi::gbm_bo) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_fd(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_fd(bo);
    }

    unsafe fn gbm_bo_get_modifier(&self, bo: *mut ffi::gbm_bo) -> u64 {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_modifier(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_modifier(bo);
    }

    unsafe fn gbm_bo_get_plane_count(&self, bo: *mut ffi::gbm_bo) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_plane_count(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_plane_count(bo);
    }

    unsafe fn gbm_bo_get_handle_for_plane(
        &self,
        bo: *mut ffi::gbm_bo,
        plane: libc::c_int,
    ) -> ffi::gbm_bo_handle {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_handle_for_plane(bo, plane);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_handle_for_plane(bo, plane);
    }

    unsafe fn gbm_bo_get_fd_for_plane(
        &self,
        bo: *mut ffi::gbm_bo,
        plane: libc::c_int,
    ) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_fd_for_plane(bo, plane);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_fd_for_plane(bo, plane);
    }

    unsafe fn gbm_bo_write(
        &self,
        bo: *mut ffi::gbm_bo,
        buf: *const libc::c_void,
        count: usize,
    ) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_write(bo, buf, count);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_write(bo, buf, count);
    }

    unsafe fn gbm_bo_set_user_data(
        &self,
        bo: *mut ffi::gbm_bo,
        data: *mut libc::c_void,
        destroy_user_data: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut ffi::gbm_bo, arg2: *mut libc::c_void),
        >,
    ) {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_set_user_data(bo, data, destroy_user_data);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_set_user_data(bo, data, destroy_user_data);
    }

    unsafe fn gbm_bo_get_user_data(&self, bo: *mut ffi::gbm_bo) -> *mut libc::c_void {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_get_user_data(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_get_user_data(bo);
    }

    unsafe fn gbm_bo_destroy(&self, bo: *mut ffi::gbm_bo) {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_bo_destroy(bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_bo_destroy(bo);
    }

    unsafe fn gbm_surface_create(
        &self,
        gbm: *mut ffi::gbm_device,
        width: u32,
        height: u32,
        format: u32,
        flags: u32,
    ) -> *mut ffi::gbm_surface {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_create(gbm, width, height, format, flags);
        #[cfg(feature = "dynamic")]
        return self
            .lib
            .gbm_surface_create(gbm, width, height, format, flags);
    }

    unsafe fn gbm_surface_create_with_modifiers(
        &self,
        gbm: *mut ffi::gbm_device,
        width: u32,
        height: u32,
        format: u32,
        modifiers: *const u64,
        count: libc::c_uint,
    ) -> *mut ffi::gbm_surface {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_create_with_modifiers(
            gbm, width, height, format, modifiers, count,
        );
        #[cfg(feature = "dynamic")]
        return self
            .lib
            .gbm_surface_create_with_modifiers(gbm, width, height, format, modifiers, count);
    }

    unsafe fn gbm_surface_create_with_modifiers2(
        &self,
        gbm: *mut ffi::gbm_device,
        width: u32,
        height: u32,
        format: u32,
        modifiers: *const u64,
        count: libc::c_uint,
        flags: u32,
    ) -> *mut ffi::gbm_surface {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_create_with_modifiers2(
            gbm, width, height, format, modifiers, count, flags,
        );
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_surface_create_with_modifiers2(
            gbm, width, height, format, modifiers, count, flags,
        );
    }

    unsafe fn gbm_surface_lock_front_buffer(
        &self,
        surface: *mut ffi::gbm_surface,
    ) -> *mut ffi::gbm_bo {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_lock_front_buffer(surface);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_surface_lock_front_buffer(surface);
    }

    unsafe fn gbm_surface_release_buffer(
        &self,
        surface: *mut ffi::gbm_surface,
        bo: *mut ffi::gbm_bo,
    ) {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_release_buffer(surface, bo);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_surface_release_buffer(surface, bo);
    }

    unsafe fn gbm_surface_has_free_buffers(&self, surface: *mut ffi::gbm_surface) -> libc::c_int {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_has_free_buffers(surface);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_surface_has_free_buffers(surface);
    }

    unsafe fn gbm_surface_destroy(&self, surface: *mut ffi::gbm_surface) {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_surface_destroy(surface);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_surface_destroy(surface);
    }

    unsafe fn gbm_format_get_name(
        &self,
        gbm_format: u32,
        desc: *mut ffi::gbm_format_name_desc,
    ) -> *mut libc::c_char {
        #[cfg(not(feature = "dynamic"))]
        return ffi::gbm_format_get_name(gbm_format, desc);
        #[cfg(feature = "dynamic")]
        return self.lib.gbm_format_get_name(gbm_format, desc);
    }
}

/// Trait for types that allow obtaining the underlying raw pointer.
pub trait AsRaw<T> {
    /// Receive a raw pointer representing this type.
    fn as_raw(&self) -> *const T;

    #[doc(hidden)]
    fn as_raw_mut(&self) -> *mut T {
        self.as_raw() as *mut _
    }
}

/// Private inner type for [`Ptr`]
///
/// This holds the raw pointer and the destructor callback for [`Ptr`].
struct PtrDrop<T>(*mut T, Option<Box<dyn FnOnce(*mut T) + Send + 'static>>);

impl<T> Drop for PtrDrop<T> {
    fn drop(&mut self) {
        (self.1.take().unwrap())(self.0);
    }
}

/// A reference-counted smart pointer with destructor callback.
///
/// This type wraps a raw pointer into a thread-safe, reference-counted struct
/// which will call a provided destructor when all references to the pointer
/// are dropped.
///
/// Raw pointers are !Send and !Sync by default in Rust, but this wrapper
/// explicitly implements Send and Sync. The gbm types used with Ptr in this
/// crate are thread-safe and therefore this is correct within the context of
/// the gbm crate.
///
/// The destructor callback takes the raw pointer as an argument. It will be
/// called when all references to the Ptr have been dropped and should perform
/// any necessary cleanup for the pointer type.
#[derive(Clone)]
pub(crate) struct Ptr<T>(Arc<PtrDrop<T>>);

// SAFETY: The types used with Ptr in this crate are all Send and Sync (namely
// gbm_device, gbm_surface and gbm_bo). Reference counting is implemented with
// the thread-safe atomic `Arc`-wrapper. The type is private and therefore
// cannot be used unsoundly by other crates.
unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}

impl<T> Ptr<T> {
    fn new<F: FnOnce(*mut T) + Send + 'static>(ptr: *mut T, destructor: F) -> Ptr<T> {
        Ptr(Arc::new(PtrDrop(ptr, Some(Box::new(destructor)))))
    }
}

impl<T> std::ops::Deref for Ptr<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

impl<T> fmt::Pointer for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Pointer::fmt(&self.0 .0, f)
    }
}

#[cfg(test)]
mod test {
    use std::os::unix::io::OwnedFd;

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    #[test]
    fn device_is_send() {
        is_send::<super::Device<std::fs::File>>();
        is_send::<super::Device<OwnedFd>>();
    }

    #[test]
    fn device_is_sync() {
        is_sync::<super::Device<std::fs::File>>();
        is_sync::<super::Device<OwnedFd>>();
    }

    #[test]
    fn surface_is_send() {
        is_send::<super::Surface<std::fs::File>>();
        is_send::<super::Surface<OwnedFd>>();
    }

    #[test]
    fn surface_is_sync() {
        is_sync::<super::Surface<std::fs::File>>();
        is_sync::<super::Surface<OwnedFd>>();
    }

    #[test]
    fn unmapped_bo_is_send() {
        is_send::<super::BufferObject<()>>();
    }

    #[test]
    fn unmapped_bo_is_sync() {
        is_sync::<super::BufferObject<()>>();
    }
}
