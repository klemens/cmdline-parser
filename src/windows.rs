#[derive(PartialEq, Eq)]
enum ParsingState {
    Normal,
    Quoted,
    QuotedEscaped,
}

pub fn parse(cmdline: &str) -> Vec<(usize, usize, String)> {
    use self::ParsingState::*;

    let mut args = vec![];
    let mut arg = String::new();
    let mut arg_start = 0;

    let mut state = Normal;

    for (i, c) in cmdline.char_indices() {
        state = match (state, c) {
            (Normal, '"') => Quoted,
            (Normal, ' ') => {
                if arg.len() > 0 {
                    args.push((arg_start, i - 1, arg.clone()));
                    arg.clear();
                }
                arg_start = i + 1;
                Normal
            },
            (Normal, _) => { arg.push(c); Normal },
            (Quoted, '"') => Normal,
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
    }

    if state == QuotedEscaped {
        arg.push('\\');
    }

    if arg.len() > 0 {
        args.push((arg_start, cmdline.len() - 1, arg));
    }

    args
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse() {
        use super::parse;

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

        // unfinished quoting
        assert_eq!(parse(r#""a"#), [(0, 1, "a".into())]);

        // unfinished escaping
        assert_eq!(parse(r#""a\"#), [(0, 2, r"a\".into())]);
        assert_eq!(parse(r#""a\""#), [(0, 3, r#"a""#.into())]);
    }
}
