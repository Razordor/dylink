# Dylink

![Crates.io](https://img.shields.io/crates/l/dylink) ![Crates.io](https://img.shields.io/crates/v/dylink) ![Crates.io](https://img.shields.io/crates/d/dylink) ![docs.rs](https://img.shields.io/docsrs/dylink) [![dylink-rs](https://github.com/Razordor/dylink/actions/workflows/rust.yml/badge.svg)](https://github.com/Razordor/dylink/actions/workflows/rust.yml) ![unsafe:yes](https://img.shields.io/badge/unsafe-yes-red)

Dylink provides a run-time dynamic linking framework for lazily evaluating shared libraries such as `.dll` files.
When functions are loaded they are evaluated through a thunk for first time calls, which loads the function from
it's respective library. Proceeding calls after initialization have no overhead or additional branching checks,
as the thunk is replaced by the loaded function.

----

Related links:

* [API Documentation](https://docs.rs/dylink)
* [Release notes](https://github.com/Razordor/dylink/releases)

## Supported platforms

Dylink has been implemented for all major platforms.

| Windows | Linux | MacOS | WASM |
|:-------:|:-----:|:-----:|------|
| YES     | YES   | YES   | NO   |

## Usage

Add this to your `Cargo.toml`

```toml
[dependencies]
dylink = "0.5"
```

## Example

Below is a basic working example on how to use the macro on windows.
For windows, the `.dll` file extension is *optional*, but still recommended.

```rust
use dylink::dylink;

#[dylink(name = "Kernel32.dll")]
extern "stdcall" {
    fn GetLastError() -> u32;
    fn SetLastError(_: u32);
}

fn main() {
   unsafe {
      SetLastError(52);
      assert_eq!(52, GetLastError());
   }
}
```

## Adv. Example

The `dylink` macro is also sophisticated enough to survive in `impl` blocks, and take advantage of the receiver argument `self`.
Unfortunately, `Self` cannot be internally inferred, so `self` without an explicit type also cannot be inferred.
The example below demonstrates how to work around these issues despite that:

```rust
use dylink::dylink;

#[derive(Debug, PartialEq)]
#[repr(transparent)]
struct Foo(u32);

impl Foo {
   #[dylink(name = "Kernel32.dll")]
   extern "stdcall" fn GetLastError() -> Foo;
   #[dylink(name = "Kernel32.dll")]
   extern "stdcall" fn SetLastError(self: Foo);
}

fn main() {
   let foo = Foo(43);
   unsafe {
      foo.SetLastError();
      assert_eq!(Foo(43), Foo::GetLastError());
   }
}
```

### License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
