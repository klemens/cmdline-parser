use std::collections::HashSet;
use std::iter::Peekable;
use std::ops::Range;
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
    separators: HashSet<char>,
}

impl<'a> Parser<'a> {
    pub fn new(cmdline: &str) -> Parser {
        Parser {
            state: ParsingState::Normal,
            cmdline: cmdline.char_indices().peekable(),
            cmdline_len: cmdline.len(),
            separators: [' '].iter().cloned().collect(),
        }
    }

    pub fn set_separators<I: IntoIterator<Item=char>>(&mut self, separators: I) {
        self.separators.clear();
        self.separators.extend(separators);
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
                    (Normal, '"') => Quoted,
                    (Normal, ref c) if self.separators.contains(c) => {
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
                    return Some((start..i, arg));
                }
            }

            if self.state == QuotedEscaped {
                arg.push('\\');
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

        // no quoting, escaping should have no effect
        assert_eq!(parse(r"arg1 arg\2 arg3\ arg4  arg5"), [
            ( 0.. 4, r"arg1".into()),
            ( 5..10, r"arg\2".into()),
            (11..16, r"arg3\".into()),
            (17..21, r"arg4".into()),
            (23..27, r"arg5".into()),
        ]);

        // quoting and escaped quotes
        assert_eq!(parse(r#""arg 1" "arg "2 "arg\3" "arg\\4" "arg\"5""#), [
            ( 0.. 7, r#"arg 1"#.into()),
            ( 8..15, r#"arg 2"#.into()),
            (16..23, r#"arg\3"#.into()),
            (24..32, r#"arg\4"#.into()),
            (33..41, r#"arg"5"#.into()),
        ]);

        // emtpy arguments
        assert_eq!(parse(r#""" """#), [(0..2, r"".into()), (3..5, r"".into())]);

        // unfinished quoting
        assert_eq!(parse(r#""a"#), [(0..2, "a".into())]);

        // unfinished escaping
        assert_eq!(parse(r#""a\"#), [(0..3, r"a\".into())]);
        assert_eq!(parse(r#""a\""#), [(0..4, r#"a""#.into())]);
    }

    #[test]
    fn multiple_separators() {
        let mut parser = super::Parser::new("arg1|arg 2:arg3");
        parser.set_separators(['|', ':'].iter().cloned());

        assert_eq!(parser.collect::<Vec<_>>(), [
            (0..4, "arg1".into()),
            (5..10, "arg 2".into()),
            (11..15, "arg3".into()),
        ]);
    }

    #[test]
    fn dynamic_separators() {
        let mut parser = super::Parser::new("arg1 arg 2:arg3");

        assert_eq!(parser.next(), Some((0..4, "arg1".into())));

        parser.set_separators([':'].iter().cloned());
        assert_eq!(parser.next(), Some((5..10, "arg 2".into())));
        assert_eq!(parser.next(), Some((11..15, "arg3".into())));
        assert_eq!(parser.next(), None);
    }
}
