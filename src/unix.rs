#[derive(PartialEq, Eq)]
enum ParsingState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
}

pub fn parse(cmdline: &str) -> Vec<(usize, usize, String)> {
    use self::ParsingState::*;

    let mut args = vec![];
    let mut arg = String::new();
    let mut arg_start = 0;

    let mut state = Normal;

    for (i, c) in cmdline.char_indices() {
        state = match (state, c) {
            (Normal, '\\') => Escaped,
            (Normal, '\'') => SingleQuoted,
            (Normal, '"') => DoubleQuoted,
            (Normal, ' ') => {
                if arg.len() > 0 {
                    args.push((arg_start, i - 1, arg.clone()));
                    arg.clear();
                }
                arg_start = i + 1;
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

