// Copyright (c) 2023 Jonathan "Razordor" Alan Thomason
use std::{cell, mem, sync};

use crate::*;

mod loader;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum LinkType {
	/// Specialization for loading vulkan functions
	Vulkan,
	/// Generalization for loading normal functions.
	Normal(&'static [&'static str]),
}

/// Fundamental data type of dylink.
///
/// This can be used safely without the dylink macro, however using the `dylink` macro should be preferred.
/// This structure can be used seperate from the dylink macro to check if the libraries exist before calling a dylink generated function.
pub struct LazyFn<F: 'static> {
	// it's imperative that LazyFn manages once, so that `LazyFn::load` is sound.
	once: sync::Once,
	// this is here to track the state of the instance.
	status: cell::UnsafeCell<Option<error::DylinkError>>,
	// The function to be called.
	// Non-function types can be stored, but obviously can't be called (call ops aren't overloaded).
	addr: cell::UnsafeCell<F>,
}

impl<F: 'static> LazyFn<F> {
	/// Initializes a `LazyFn` object with all the necessary information for `LazyFn::link` to work.
	/// # Panic
	/// Type `F` must be the same size as a [function pointer](fn).
	#[inline]
	pub const fn new(thunk: F) -> Self {
		// In a const context this assert will be optimized out.
		assert!(mem::size_of::<FnPtr>() == mem::size_of::<F>());
		Self {
			addr: cell::UnsafeCell::new(thunk),
			once: sync::Once::new(),
			status: cell::UnsafeCell::new(None),
		}
	}

	// This is intentionally non-generic to reduce code bloat, and the function overhead has been found to be relatively trivial.
	/// If successful, stores address and returns it.
	pub fn load(&self, fn_name: &'static ffi::CStr, link_ty: LinkType) -> Result<&F> {
		let str_name = fn_name.to_str().unwrap();
		self.once.call_once(|| unsafe {
			let maybe = match link_ty {
				LinkType::Vulkan => {
					match fn_name.to_str().unwrap() {
						"vkGetInstanceProcAddr" => Ok(mem::transmute::<
							unsafe extern "system" fn(
								instance: ffi::VkInstance,
								pName: *const ffi::c_char,
							) -> Option<FnPtr>,
							FnPtr,
						>(loader::vkGetInstanceProcAddr)),
						"vkGetDeviceProcAddr" => Ok(mem::transmute::<
							unsafe extern "system" fn(
								device: ffi::VkDevice,
								name: *const ffi::c_char,
							) -> Option<FnPtr>,
							FnPtr,
						>(loader::vkGetDeviceProcAddr)),
						_ => {
							let device_read_lock =
								VK_DEVICE.read().expect("failed to get read lock");
							match device_read_lock.iter().find_map(|device| {
								loader::vkGetDeviceProcAddr(*device, fn_name.as_ptr() as *const _)
							}) {
								Some(addr) => Ok(addr),
								None => {
									mem::drop(device_read_lock);
									let instance_read_lock =
										VK_INSTANCE.read().expect("failed to get read lock");
									// check other instances if fails in case one has a higher available version number
									match instance_read_lock.iter().find_map(|instance| {
										loader::vkGetInstanceProcAddr(*instance, fn_name.as_ptr())
									}) {
										Some(addr) => Ok(addr),
										None => loader::vkGetInstanceProcAddr(
											ffi::VkInstance(std::ptr::null()),
											fn_name.as_ptr(),
										)
										.ok_or(error::DylinkError::new(
											Some(str_name),
											ErrorKind::FnNotFound,
										)),
									}
								}
							}
						}
					}
				}
				LinkType::Normal(lib_list) => {
					let default_error = {
						let (subject, kind) = if lib_list.len() > 1 {
							(None, ErrorKind::ListNotFound)
						} else {
							(Some(lib_list[0]), ErrorKind::LibNotFound)
						};
						error::DylinkError::new(subject, kind)
					};
					let mut result = Err(default_error);
					for lib_name in lib_list {
						match loader::loader(ffi::OsStr::new(lib_name), str_name) {
							Ok(addr) => {
								result = Ok(addr);
								// success! lib and function retrieved!
								break;
							}
							Err(err) => {
								if let ErrorKind::FnNotFound = err.kind() {
									result = Err(err);
									// lib detected, but function failed to load
									break;
								}
							}
						}
					}
					result
				}
			};
			match maybe {
				Ok(addr) => {
					cell::UnsafeCell::raw_get(&self.addr).write(mem::transmute_copy(&addr));
				}
				Err(err) => {
					cell::UnsafeCell::raw_get(&self.status).write(Some(err));
				}
			}
		});
		// `call_once` is blocking, so `self.status` is read-only
		// by this point. Race conditions shouldn't occur.
		match unsafe { (*self.status.get()).clone() } {
			None => Ok(self.as_ref()),
			Some(err) => Err(err),
		}
	}
}

unsafe impl<F: 'static> Send for LazyFn<F> {}
unsafe impl<F: 'static> Sync for LazyFn<F> {}

impl<F: 'static> std::ops::Deref for LazyFn<F> {
	type Target = F;

	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}

impl<F: 'static> std::convert::AsRef<F> for LazyFn<F> {
	// `addr` is never uninitialized, so `unwrap_unchecked` is safe.
	#[inline]
	fn as_ref(&self) -> &F {
		unsafe { self.addr.get().as_ref().unwrap_unchecked() }
	}
}
