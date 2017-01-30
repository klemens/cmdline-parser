use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ParsingState {
    Normal,
    Quoted,
    QuotedEscaped,
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
            let mut was_quoted = false;

            for (i, c) in &mut self.cmdline {
                self.state = match (self.state, c) {
                    (Normal, '"') => Quoted,
                    (Normal, ' ') => {
                        if arg.len() > 0 || was_quoted {
                            yield_value = true;
                        } else {
                            start = i + 1;
                        }
                        Normal
                    },
                    (Normal, _) => { arg.push(c); Normal },
                    (Quoted, '"') => { was_quoted = true; Normal },
                    (Quoted, '\\') => QuotedEscaped,
                    (Quoted, _) => { arg.push(c); Quoted },
                    (QuotedEscaped, '"') |
                    (QuotedEscaped, '\\') => { arg.push(c); Quoted },
                    (QuotedEscaped, _) => {
                        arg.push('\\');
                        arg.push(c);
                        Quoted
                    },
                };

                if yield_value {
                    return Some((start, i - 1, arg));
                }
            }

            if self.state == QuotedEscaped {
                arg.push('\\');
            }

            if arg.len() > 0 || was_quoted {
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
        // no quoting, escaping should have no effect
        assert_eq!(parse(r"arg1 arg\2 arg3\ arg4  arg5"), [
            ( 0,  3, r"arg1".into()),
            ( 5,  9, r"arg\2".into()),
            (11, 15, r"arg3\".into()),
            (17, 20, r"arg4".into()),
            (23, 26, r"arg5".into()),
        ]);

        // quoting and escaped quotes
        assert_eq!(parse(r#""arg 1" "arg "2 "arg\3" "arg\\4" "arg\"5""#), [
            ( 0,  6, r#"arg 1"#.into()),
            ( 8, 14, r#"arg 2"#.into()),
            (16, 22, r#"arg\3"#.into()),
            (24, 31, r#"arg\4"#.into()),
            (33, 40, r#"arg"5"#.into()),
        ]);

        // emtpy arguments
        assert_eq!(parse(r#""" """#), [(0, 1, r"".into()), (3, 4, r"".into())]);

        // unfinished quoting
        assert_eq!(parse(r#""a"#), [(0, 1, "a".into())]);

        // unfinished escaping
        assert_eq!(parse(r#""a\"#), [(0, 2, r"a\".into())]);
        assert_eq!(parse(r#""a\""#), [(0, 3, r#"a""#.into())]);
    }
}
