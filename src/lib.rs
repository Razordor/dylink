// Copyright (c) 2022 Jonathan "Razordor" Alan Thomason
#![cfg_attr(feature = "opaque_types", feature(extern_types))]
#![allow(clippy::missing_safety_doc)]
use std::{collections::HashSet, sync};

use once_cell::sync::Lazy;

pub mod error;
pub mod lazyfn;

// TODO: make this work through more than just windows
pub mod loader;

// This global is read every time a vulkan function is called for the first time,
// which occurs through `LazyFn::link`.
static VK_INSTANCE: sync::RwLock<Lazy<HashSet<VkInstance>>> =
	sync::RwLock::new(Lazy::new(|| HashSet::new()));

static VK_DEVICE: sync::RwLock<Lazy<HashSet<VkDevice>>> =
	sync::RwLock::new(Lazy::new(|| HashSet::new()));

/// Used as a placeholder function pointer. This should **NEVER** be called directly,
/// and promptly cast into the correct function pointer type.
pub type FnPtr = unsafe extern "system" fn() -> isize;
/// The result of a dylink function
pub type Result<T> = std::result::Result<T, error::DylinkError>;

// FIXME: when extern types are stablized they must replace the `c_void` variation

// extern "C" {
// 	type VkInstance_T;
// 	type VkDevice_T;
// }

// #[derive(Clone, Copy, Eq, Hash, PartialEq)]
// #[repr(transparent)]
// pub struct VkInstance(pub(crate) *const VkInstance_T);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct VkInstance(pub(crate) *const std::ffi::c_void);

// pretend VkInstance is not a pointer. dylink never dereferences the contents (because it can't),
// so there shouldn't be aliasing problems.
unsafe impl Sync for VkInstance {}
unsafe impl Send for VkInstance {}

// #[derive(Clone, Copy, Eq, Hash, PartialEq)]
// #[repr(transparent)]
// pub struct VkDevice(pub(crate) *const VkDevice_T);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct VkDevice(pub(crate) *const std::ffi::c_void);

// pretend VkInstance is not a pointer. dylink never dereferences the contents (because it can't),
// so there shouldn't be aliasing problems.
unsafe impl Sync for VkDevice {}
unsafe impl Send for VkDevice {}

pub struct Global;
impl Global {
	// This is safe since vulkan will just discard garbage values
	/// Adds an instance to the internal HashSet.
	///
	/// Returns whether the instance was newly inserted. That is:
	///
	/// *    If the set did not previously contain this value, `true` is returned.
	/// *    If the set already contained this value, `false` is returned.
	///
	/// *note: This function returns `false` if the instance is valid and defined through dylink.*
	pub fn insert_instance(&self, instance: VkInstance) -> bool {
		let mut write_lock = VK_INSTANCE.write().unwrap();
		write_lock.insert(instance)
	}

	/// Removes an instance from the set. Returns whether the instance was present in the set.
	/// # Safety
	/// Using this function may break dylink's checked lifetimes!
	pub unsafe fn remove_instance(&self, instance: &VkInstance) -> bool {
		let mut write_lock = VK_INSTANCE.write().unwrap();
		write_lock.remove(instance)
	}

	// This is safe since vulkan will just discard garbage values
	/// Adds a device to the internal HashSet.
	///
	/// Returns whether the device was newly inserted. That is:
	///
	/// *    If the set did not previously contain this value, `true` is returned.
	/// *    If the set already contained this value, `false` is returned.
	///
	/// *note: This function returns `false` if the device is valid and defined through dylink.*
	pub fn insert_device(&self, device: VkDevice) -> bool {
		let mut write_lock = VK_DEVICE.write().unwrap();
		write_lock.insert(device)
	}

	/// Removes a device from the set. Returns whether the value was present in the set.
	/// # Safety
	/// Using this function may break dylink's checked lifetimes!
	pub unsafe fn remove_device(&self, device: &VkDevice) -> bool {
		let mut write_lock = VK_DEVICE.write().unwrap();
		write_lock.remove(device)
	}
}
