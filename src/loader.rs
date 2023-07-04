// Copyright (c) 2023 Jonathan "Razordor" Alan Thomason

use crate::*;
use std::ffi;
use std::io;


#[cfg(any(windows, unix, doc))]
mod self_loader;
#[cfg(any(windows, unix, doc))]
mod sys_loader;

/// This trait is similar to the `Drop` trait, which frees resources.
/// Unlike the `Drop` trait, `Close` must assume there side affects when closing a library.
/// As a consequence of these side affects `close` is marked as `unsafe`.
/// 
/// This trait should not be used directly, and instead be used in conjunction with `CloseableLibrary`,
/// so that the lifetimes of retrieved symbols are not invalidated.
#[cfg(any(feature = "close", doc))]
pub trait Close {
	unsafe fn close(self) -> io::Result<()>;
}


/// Used to specify the run-time linker loader constraint for [`Library`]
pub unsafe trait Loader: Send {
	fn is_invalid(&self) -> bool;
	unsafe fn load_library(lib_name: &'static ffi::CStr) -> Self;
	unsafe fn find_symbol(&self, fn_name: &'static ffi::CStr) -> FnAddr;
}

/// A system library loader.
/// 
/// This is a basic library loader primitive designed to be used with [`Library`].
#[cfg(any(windows, unix, doc))]
pub struct SystemLoader(*mut core::ffi::c_void);


/// `SelfLoader` is a special structure that retrieves symbols from libraries already
/// loaded before hand such as `libc` or `kernel32`
///
/// # Example
///
/// ```rust
/// use dylink::*;
/// use std::ffi::{c_char, c_int, CStr};
///
/// static LIBC_LIB: Library<SelfLoader, 1> = Library::new([
///   // dummy value for Library
///   unsafe { CStr::from_bytes_with_nul_unchecked(b"libc\0") }
/// ]);
///
/// #[dylink(library=LIBC_LIB)]
/// extern "C" {
/// 	fn atoi(s: *const c_char) -> c_int;
/// }
///
/// # #[cfg(unix)] {
/// let five = unsafe { atoi(b"5\0".as_ptr().cast()) };
/// assert_eq!(five, 5);
/// # }
/// ```
#[cfg(any(windows, unix, doc))]
pub struct SelfLoader(*mut core::ffi::c_void);
