use std::iter::Peekable;
use std::ops::Range;
use std::str::CharIndices;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ParsingState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
}

pub struct Parser<'a> {
    state: ParsingState,
    cmdline: Peekable<CharIndices<'a>>,
    cmdline_len: usize,
}

impl<'a> Parser<'a> {
    pub fn new(cmdline: &str) -> Parser {
        Parser {
            state: ParsingState::Normal,
            cmdline: cmdline.char_indices().peekable(),
            cmdline_len: cmdline.len(),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = (Range<usize>, String);

    fn next(&mut self) -> Option<Self::Item> {
        use self::ParsingState::*;

        let mut arg = String::new();

        if let Some(&(mut start, _)) = self.cmdline.peek() {
            let mut yield_value = false;
            let mut was_quoted = false;

            for (i, c) in &mut self.cmdline {
                self.state = match (self.state, c) {
                    (Normal, '\\') => Escaped,
                    (Normal, '\'') => SingleQuoted,
                    (Normal, '"') => DoubleQuoted,
                    (Normal, ' ') => {
                        if arg.len() > 0 || was_quoted {
                            yield_value = true;
                        } else {
                            start = i + 1;
                        }
                        Normal
                    },
                    (Normal, _) |
                    (Escaped, _) => { arg.push(c); Normal },
                    (SingleQuoted, '\'') => { was_quoted = true; Normal },
                    (SingleQuoted, _) => { arg.push(c); SingleQuoted },
                    (DoubleQuoted, '"') => { was_quoted = true; Normal },
                    (DoubleQuoted, '\\') => DoubleQuotedEscaped,
                    (DoubleQuoted, _) |
                    (DoubleQuotedEscaped, '"') |
                    (DoubleQuotedEscaped, '\\') => { arg.push(c); DoubleQuoted },
                    (DoubleQuotedEscaped, _) => {
                        arg.push('\\');
                        arg.push(c);
                        DoubleQuoted
                    },
                };

                if yield_value {
                    return Some((start..i, arg));
                }
            }

            if arg.len() > 0 || was_quoted {
                return Some((start..self.cmdline_len, arg));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parser() {
        let parse = |cmd| -> Vec<_> { super::Parser::new(cmd).collect() };

        // no quoting and simple escaping
        assert_eq!(parse(r"arg1 arg\2 arg3\ arg4  arg5 \a\r\g\\6"), [
            ( 0.. 4, r"arg1".into()),
            ( 5..10, r"arg2".into()),
            (11..21, r"arg3 arg4".into()),
            (23..27, r"arg5".into()),
            (28..37, r"arg\6".into()),
        ]);

        // single quoting
        assert_eq!(parse(r#"'arg 1' 'arg '2 'arg\3' 'arg\\4' 'arg"5' '\'arg6"#), [
            ( 0.. 7, r#"arg 1"#.into()),
            ( 8..15, r#"arg 2"#.into()),
            (16..23, r#"arg\3"#.into()),
            (24..32, r#"arg\\4"#.into()),
            (33..40, r#"arg"5"#.into()),
            (41..48, r#"\arg6"#.into()),
        ]);

        // double quoting
        assert_eq!(parse(r#""arg 1" "arg "2 "arg\3" "arg\\4" "arg'5" "arg\"6""#), [
            ( 0.. 7, r#"arg 1"#.into()),
            ( 8..15, r#"arg 2"#.into()),
            (16..23, r#"arg\3"#.into()),
            (24..32, r#"arg\4"#.into()),
            (33..40, r#"arg'5"#.into()),
            (41..49, r#"arg"6"#.into()),
        ]);

        // emtpy arguments
        assert_eq!(parse(r#"'' """#), [(0..2, r"".into()), (3..5, r"".into())]);

        // unfinished escaping
        assert_eq!(parse(r#"a\"#), [(0..2, r"a".into())]);

        // unfinished quoting (causes an error in a real shell)
        assert_eq!(parse(r#""a"#), [(0..2, "a".into())]);
        assert_eq!(parse(r#"'a"#), [(0..2, "a".into())]);
    }
}

