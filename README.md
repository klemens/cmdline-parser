# cmdline-parser

[Documentation](https://docs.rs/cmdline-parser)

Library to parse cmdlines in a way that resembles the platform default.
Supports cmd and bash-like parsing.

# Example

```rust
extern crate cmdline_parser;
use cmdline_parser::Parser;

fn main() {
    let mut parser = Parser::new(r#"mv "my file" project/"#);

    assert_eq!(parser.next(), Some((0..2, "mv".into())));
    assert_eq!(parser.next(), Some((3..12, "my file".into())));
    assert_eq!(parser.next(), Some((13..21, "project/".into())));
    assert_eq!(parser.next(), None);
}
```

# Licence

This library is licensed under the terms of the MIT and Apache 2.0 licences.
