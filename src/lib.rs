pub mod unix;
pub mod windows;

#[cfg(unix)]
pub use unix::parse;
#[cfg(windows)]
pub use windows::parse;
