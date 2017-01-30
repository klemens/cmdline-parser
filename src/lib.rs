pub mod unix;
pub mod windows;

#[cfg(unix)]
pub use unix::Parser;
#[cfg(windows)]
pub use windows::Parser;
