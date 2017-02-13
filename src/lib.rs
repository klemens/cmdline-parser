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

/// Parse the given string as a single argument.
///
/// Resolves quoting and escaping, but does not split arguments.
pub fn parse_single(argument: &str) -> String {
    let mut parser = Parser::new(argument);
    parser.set_separators(std::iter::empty());

    parser.nth(0).map(|(_, arg)| arg).unwrap_or("".into())
}
