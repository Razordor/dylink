// Copyright (c) 2023 Jonathan "Razordor" Alan Thomason

use crate::lazyfn;
use crate::{FnPtr, LinkType};
use std::ffi::CStr;
use std::sync::atomic::Ordering;
use std::{ffi, mem};

// dylink_macro internally uses dylink as it's root namespace,
// but since we are in dylink the namespace is actually named `self`.
// this is just here to resolve the missing namespace issue.
extern crate self as dylink;

#[doc(hidden)]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkInstance(*mut ffi::c_void);
unsafe impl Sync for VkInstance {}
unsafe impl Send for VkInstance {}

#[doc(hidden)]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkDevice(*mut ffi::c_void);
unsafe impl Sync for VkDevice {}
unsafe impl Send for VkDevice {}

// Windows and Linux are fully tested and useable as of this comment.
// MacOS should theoretically work, but it's untested.
// This function is in itself an axiom of the vulkan specialization.
//
// Do not add `strip` here since loader::lazyfn::vulkan_loader needs it as a function pointer.
#[cfg_attr(windows, crate::dylink(name = "vulkan-1.dll"))]
#[cfg_attr(
	all(unix, not(target_os = "macos")),
	crate::dylink(any(name = "libvulkan.so.1", name = "libvulkan.so"))
)]
#[cfg_attr(
	target_os = "macos",
	crate::dylink(any(
		name = "libvulkan.dylib",
		name = "libvulkan.1.dylib",
		name = "libMoltenVK.dylib"
	))
)]
extern "system" {
	pub(crate) fn vkGetInstanceProcAddr(
		instance: VkInstance,
		pName: *const ffi::c_char,
	) -> Option<FnPtr>;
}

#[allow(non_camel_case_types)]
pub(crate) type PFN_vkGetDeviceProcAddr =
	unsafe extern "system" fn(VkDevice, *const ffi::c_char) -> Option<FnPtr>;

// vkGetDeviceProcAddr must be implemented manually to avoid recursion
#[allow(non_snake_case)]
#[inline]
pub(crate) unsafe extern "system" fn vkGetDeviceProcAddr(
	device: VkDevice,
	name: *const ffi::c_char,
) -> Option<FnPtr> {
	unsafe extern "system" fn initial_fn(
		device: VkDevice,
		name: *const ffi::c_char,
	) -> Option<FnPtr> {
		DEVICE_PROC_ADDR.once.get_or_init(|| {
			let read_lock = crate::VK_INSTANCE
				.read()
				.expect("Dylink Error: failed to get read lock");
			// check other instances if fails in case one has a higher available version number
			let fn_ptr = read_lock
				.iter()
				.find_map(|instance| {
					vkGetInstanceProcAddr(
						*instance,
						b"vkGetDeviceProcAddr\0".as_ptr() as *const ffi::c_char,
					)
				})
				.expect("Dylink Error: failed to load `vkGetDeviceProcAddr`.");

			*DEVICE_PROC_ADDR.addr.get() = mem::transmute(fn_ptr);
			DEVICE_PROC_ADDR
				.addr_ptr
				.store(DEVICE_PROC_ADDR.addr.get(), Ordering::Relaxed);
		});
		DEVICE_PROC_ADDR(device, name)
	}

	pub(crate) static DEVICE_PROC_ADDR: lazyfn::LazyFn<PFN_vkGetDeviceProcAddr> =
		lazyfn::LazyFn::new(
			&(initial_fn as PFN_vkGetDeviceProcAddr),
			unsafe { CStr::from_bytes_with_nul_unchecked(b"vkGetDeviceProcAddr\0") },
			LinkType::Vulkan,
		);
	DEVICE_PROC_ADDR(device, name)
}

pub(crate) unsafe fn vulkan_loader(fn_name: &ffi::CStr) -> Option<FnPtr> {
	let mut maybe_fn = crate::VK_DEVICE
		.read()
		.expect("failed to get read lock")
		.iter()
		.find_map(|device| vkGetDeviceProcAddr(*device, fn_name.as_ptr() as *const ffi::c_char));
	maybe_fn = match maybe_fn {
		Some(addr) => return Some(addr),
		None => crate::VK_INSTANCE
			.read()
			.expect("failed to get read lock")
			.iter()
			.find_map(|instance| {
				vkGetInstanceProcAddr(*instance, fn_name.as_ptr() as *const ffi::c_char)
			}),
	};
	match maybe_fn {
		Some(addr) => Some(addr),
		None => vkGetInstanceProcAddr(
			VkInstance(std::ptr::null_mut()),
			fn_name.as_ptr() as *const ffi::c_char,
		),
	}
}
