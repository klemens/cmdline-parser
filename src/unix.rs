use std::iter::Peekable;
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
    type Item = (usize, usize, String);

    fn next(&mut self) -> Option<Self::Item> {
        use self::ParsingState::*;

        let mut arg = String::new();

        if let Some(&(mut start, _)) = self.cmdline.peek() {
            let mut yield_value = false;
            for (i, c) in &mut self.cmdline {
                self.state = match (self.state, c) {
                    (Normal, '\\') => Escaped,
                    (Normal, '\'') => SingleQuoted,
                    (Normal, '"') => DoubleQuoted,
                    (Normal, ' ') => {
                        if arg.len() > 0 {
                            yield_value = true;
                        } else {
                            start = i + 1;
                        }
                        Normal
                    },
                    (Normal, _) |
                    (Escaped, _) => { arg.push(c); Normal },
                    (SingleQuoted, '\'') => Normal,
                    (SingleQuoted, _) => { arg.push(c); SingleQuoted },
                    (DoubleQuoted, '"') => Normal,
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
                    return Some((start, i - 1, arg));
                }
            }

            if arg.len() > 0 {
                return Some((start, self.cmdline_len - 1, arg));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    pub fn parse(cmdline: &str) -> Vec<(usize, usize, String)> {
        super::Parser::new(cmdline).collect()
    }

    #[test]
    fn parser() {
        // no quoting and simple escaping
        assert_eq!(parse(r"arg1 arg\2 arg3\ arg4  arg5 \a\r\g\\6"), [
            ( 0,  3, r"arg1".into()),
            ( 5,  9, r"arg2".into()),
            (11, 20, r"arg3 arg4".into()),
            (23, 26, r"arg5".into()),
            (28, 36, r"arg\6".into()),
        ]);

        // single quoting
        assert_eq!(parse(r#"'arg 1' 'arg '2 'arg\3' 'arg\\4' 'arg"5' '\'arg6"#), [
            ( 0,  6, r#"arg 1"#.into()),
            ( 8, 14, r#"arg 2"#.into()),
            (16, 22, r#"arg\3"#.into()),
            (24, 31, r#"arg\\4"#.into()),
            (33, 39, r#"arg"5"#.into()),
            (41, 47, r#"\arg6"#.into()),
        ]);

        // double quoting
        assert_eq!(parse(r#""arg 1" "arg "2 "arg\3" "arg\\4" "arg'5" "arg\"6""#), [
            ( 0,  6, r#"arg 1"#.into()),
            ( 8, 14, r#"arg 2"#.into()),
            (16, 22, r#"arg\3"#.into()),
            (24, 31, r#"arg\4"#.into()),
            (33, 39, r#"arg'5"#.into()),
            (41, 48, r#"arg"6"#.into()),
        ]);

        // unfinished escaping
        assert_eq!(parse(r#"a\"#), [(0, 1, r"a".into())]);

        // unfinished quoting (causes an error in a real shell)
        assert_eq!(parse(r#""a"#), [(0, 1, "a".into())]);
        assert_eq!(parse(r#"'a"#), [(0, 1, "a".into())]);
    }
}

