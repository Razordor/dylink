use dylink::*;

#[cfg(windows)]
static KERNEL32: Library<SystemLoader> = Library::new(&["Kernel32.dll"]);

#[cfg(target_os = "linux")]
static LIB_X11: CloseableLibrary<SystemLoader> = CloseableLibrary::new(&["libX11.so.6"]);

#[cfg(windows)]
#[test]
fn test_win32_kernel32() {
	#[dylink(library = KERNEL32)]
	extern "stdcall" {
		fn SetLastError(_: u32);
	}

	// macro output: function
	#[dylink(library = KERNEL32)]
	extern "system" fn GetLastError() -> u32;

	unsafe {
		SetLastError(53);
		assert_eq!(GetLastError(), 53);
	}
}

#[cfg(windows)]
#[test]
fn test_win32_impl() {
	#[repr(transparent)]
	struct Foo(u32);

	impl Foo {
		#[dylink(library = KERNEL32, link_name = "SetLastError")]
		extern "stdcall" fn set_last_error(self: Foo);

		#[dylink(library = KERNEL32)]
		extern "system" fn GetLastError() -> Foo;
	}
	let is_loaded = force(&KERNEL32).unwrap();
	assert!(is_loaded);

	let foo = Foo(23);
	unsafe {
		foo.set_last_error();
		assert!(Foo::GetLastError().0 == 23)
	}
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_x11() {
	use std::ffi::{c_char, c_void, CStr};

	#[repr(transparent)]
	struct Display(*const c_void);

	#[dylink(library = LIB_X11)]
	extern "C" {
		fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
		fn XCloseDisplay(display: *mut Display);
	}

	unsafe {
		let display_name = CStr::from_bytes_until_nul(b"foo\0").unwrap();
		let disp = XOpenDisplay(display_name.as_ptr());
		if !disp.is_null() {
			println!("display created successfully.\nnow destroying...");
			XCloseDisplay(disp);
		}
		LIB_X11
			.lock()
			.expect("failed to get lock")
			.close()
			.expect("close failed");
	}
}
