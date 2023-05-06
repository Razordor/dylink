#[cfg(windows)]
#[test]
fn test_win32_kernel32() {
	use dylink::dylink;
	// macro output: static variable
	#[dylink(name = "Kernel32.dll", strip = true)]
	extern "stdcall" {
		fn SetLastError(_: u32);		
	}

	// macro output: function
	#[dylink(name = "Kernel32.dll", strip = false)]
	extern {
		fn GetLastError() -> u32;
	}
	
	unsafe {
		// static variable has crappy documentation, but can be use for library induction.
		match SetLastError.try_link() {
			Ok(f) => f(53),
			Err(e) => panic!("{}", e),
		}
		assert_eq!(GetLastError(), 53);
	}
}

// tbh I don't know why this test passes.
#[cfg(windows)]
#[test]
fn test_win32_lifetimes() {
	use dylink::LazyFn;
	use std::ops::Deref;

	extern "stdcall" fn foo() -> u32 {
		0
	}
	type PfnTy = extern "stdcall" fn() -> u32;

	let lazyfn = LazyFn::<PfnTy>::new(
		&(foo as PfnTy),
		"SetLastError",
		dylink::LinkType::System(&["Kernel32.dll"]),
	);
	// `deref` isn't suppose to be used this way, but if
	// it is used, this test will check if it's valid.
	let old_ref = lazyfn.deref();
	let new_addr = lazyfn.try_link().unwrap();

	assert_eq!(*old_ref, foo as PfnTy);
	assert_ne!(new_addr, foo as PfnTy);
	assert_ne!(lazyfn.deref(), old_ref);
}

#[cfg(unix)]
#[test]
fn test_linux_libc() {
	use std::ffi::c_double;
	use dylink::dylink;
	#[dylink(any(name = "libc.so.6", name = "/lib/x86_64-linux-gnu/libc.so", name = "libc.so"))]
	extern "C" {
		fn floor(_: c_double) -> c_double;
	}

	//#[dylink(name = "/lib/x86_64-linux-gnu/libc.so")]
	//extern {
	//	fn ceil(_: c_double) -> c_double;
	//}
	unsafe {
		assert!(floor(10.6) == 10.);
		//assert!(ceil(10.6) == 11.);
	}
}