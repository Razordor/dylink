// Copyright (c) 2023 Jonathan "Razordor" Alan Thomason

use crate::*;
use core::ffi;

#[cfg(feature = "std")]
mod self_loader;
#[cfg(feature = "std")]
mod sys_loader;

#[doc(hidden)]
pub trait FnPtr: Copy + Clone {}
impl<T: Copy + Clone> FnPtr for T {}

pub trait LibHandle: Send {
	fn is_invalid(&self) -> bool;
}

/// Used to specify the run-time linker loader constraint for [LazyLib]
///
/// This trait must never panic, or a potential deadlock may occur when used with [LazyLib].
pub trait Loader
where
	Self::Handle: LibHandle,
{
	type Handle;
	fn load_lib(lib_name: &'static ffi::CStr) -> Self::Handle;
	fn load_sym(lib_handle: &Self::Handle, fn_name: &'static ffi::CStr) -> FnAddr;
}

/// Default system loader used in [LazyLib]
#[cfg(feature = "std")]
pub struct SysLoader;

/// `SelfLoader` is a special structure that retrieves symbols from libraries already
/// loaded before hand such as `libc` or `kernel32`
///
/// # Example
///
/// ```rust
/// use dylink::*;
/// use std::ffi::{c_char, c_int, CStr};
///
/// static LIBC_LIB: LazyLib<SelfLoader, 1> = LazyLib::new([
///   // dummy value for LazyLib
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
#[cfg(feature = "std")]
pub struct SelfLoader;
