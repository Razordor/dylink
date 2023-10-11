#[cfg(any(unix, docsrs))]
pub mod unix;
#[cfg(any(windows, docsrs))]
pub mod windows;

pub(crate) type Handle = *mut crate::Lib;
