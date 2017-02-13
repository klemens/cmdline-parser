//! Parse cmdlines in a way that resembles the platform default.
//!
//! # Example
//!
//! ```rust
//! use cmdline_parser::Parser;
//!
//! let mut parser = Parser::new(r#"mv "my file" project/"#);
//!
//! assert_eq!(parser.next(), Some((0..2, "mv".into())));
//! assert_eq!(parser.next(), Some((3..12, "my file".into())));
//! assert_eq!(parser.next(), Some((13..21, "project/".into())));
//! assert_eq!(parser.next(), None);
//! ```

pub mod unix;
pub mod windows;

#[cfg(unix)]
pub use unix::Parser;
#[cfg(windows)]
pub use windows::Parser;
