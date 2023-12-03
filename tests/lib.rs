mod linux;
mod macos;
mod unix;
mod windows;

use dylink::*;

#[test]
fn test_try_clone() {
	let lib = Library::this();
	let other = lib.try_clone().unwrap();
	assert_eq!(lib.to_header(), other.to_header());
	let t = std::thread::spawn(move || {
		println!("other: {:?}", other);
	});
	t.join().unwrap();
	println!("lib: {:?}", lib);
}

#[test]
fn test_iter_images() {
	let images = img::Images::now().unwrap();
	for weak in images {
		print!("weak addr: {:p}, ", weak.to_ptr());
		if let Some(dylib) = weak.upgrade() {
			let hdr = dylib.to_header().unwrap();
			if let Ok(path) = hdr.path() {
				println!("upgraded = {}", path.display());
				assert_eq!(path, dylib.to_header().unwrap().path().unwrap());
			}
			assert_eq!(unsafe { weak.to_ptr().as_ref() }, dylib.to_header());
		} else if let Some(path) = weak.path() {
			println!("upgrade failed = {}", path.display());
		}
	}
}

// test to see if there are race conditions when getting a path.
#[test]
fn test_path_soundness() {
	use dylink::img::Images;
	let images = Images::now().unwrap();
	let mut vlib = vec![];
	for img in images {
		if let Some(val) = img.upgrade() {
			vlib.push(val)
		}
	}
	let t = std::thread::spawn(|| {
		let images = Images::now().unwrap();
		let mut other_vlib = vec![];
		for img in images {
			if let Some(val) = img.upgrade() {
				other_vlib.push(val)
			}
		}
		for lib in other_vlib.drain(0..) {
			let _ = lib.try_clone().unwrap();
		}
	});
	for lib in vlib.drain(0..) {
		let _ = lib.try_clone().unwrap();
	}
	t.join().unwrap();
}

#[test]
fn test_hdr_magic() {
	let images = img::Images::now().unwrap();
	for img in images {
		let maybe_hdr = unsafe { img.to_ptr().as_ref() };
		let Some(hdr) = maybe_hdr else {
			continue;
		};
		let magic = hdr.magic();
		if cfg!(windows) {
			assert!(magic == [b'M', b'Z'] || magic == [b'Z', b'M'])
		} else if cfg!(target_os = "macos") {
			const MH_MAGIC: u32 = 0xfeedface;
			const MH_MAGIC_64: u32 = 0xfeedfacf;
			assert!(magic == MH_MAGIC.to_le_bytes() || magic == MH_MAGIC_64.to_le_bytes())
		} else if cfg!(unix) {
			const EI_MAG: [u8; 4] = [0x7f, b'E', b'L', b'F'];
			assert_eq!(magic, EI_MAG);
		}
	}
}

#[test]
fn test_hdr_bytes() {
	let images = img::Images::now().unwrap();
	for img in images {
		let maybe_hdr = unsafe { img.to_ptr().as_ref() };
		let Some(hdr) = maybe_hdr else {
			continue;
		};
		let bytes = hdr.to_bytes().unwrap();
		assert!(bytes.len() > 0);
		let _ = bytes[bytes.len() - 1];
	}
}

#[test]
fn test_hdr_path() {
	let images = img::Images::now().unwrap();
	for img in images {
		let maybe_hdr = unsafe { img.to_ptr().as_ref() };
		let Some(hdr) = maybe_hdr else {
			continue;
		};
		if let Some(path) = img.path() {
			assert_eq!(path, hdr.path().unwrap());
		}
	}
}
