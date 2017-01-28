pub mod windows;

#[cfg(windows)]
pub use windows::parse;
