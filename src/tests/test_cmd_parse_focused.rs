use super::*;
use crate::tests::test_fixtures::globals;

unsafe fn parse(input: &[u8]) -> cmd_parse_result {
    unsafe {
        let mut pi = cmd_parse_input {
            flags: CMD_PARSE_PARSEONLY | CMD_PARSE_NOALIAS,
            ..Default::default()
        };
        cmd_parse_from_buffer(input, Some(&mut pi))
    }
}

#[test]
fn grammar_accepts_commands_separators_quotes_and_comments() {
    let _guard = globals();
    unsafe {
        for input in [
            b"display-message hello".as_slice(),
            b"display-message one; display-message two\n",
            b"# comment\ndisplay-message 'one two' # tail\n",
            b"display-message \"double quoted\"",
            b"display-message one\\ two",
            b"display-message first\\\nsecond",
            b"display-message ''",
            b"display-message '#literal;literal'",
        ] {
            let result = parse(input);
            assert_eq!(
                result.status, CMD_PARSE_SUCCESS,
                "{:?}: {:?}",
                input, result.error
            );
            assert!(result.cmdlist.is_some());
        }
        let empty = parse(b"");
        assert_eq!(empty.status, CMD_PARSE_SUCCESS);
    }
}

#[test]
fn grammar_reports_unterminated_and_invalid_constructs() {
    let _guard = globals();
    unsafe {
        for input in [
            b"%else\ndisplay-message x\n".as_slice(),
            b"%endif\n",
            b"%if 1\ndisplay-message x\n",
            b"%if\n%endif\n",
        ] {
            let result = parse(input);
            assert_eq!(result.status, CMD_PARSE_ERROR, "{:?}", input);
            assert!(result.error.is_some());
        }
    }
}

#[test]
fn conditional_directives_keep_only_active_grammar_branches() {
    let _guard = globals();
    unsafe {
        for input in [
            b"%if 1\ndisplay-message yes\n%else\ndisplay-message no\n%endif\n".as_slice(),
            b"%if 0\ndisplay-message no\n%elif 1\ndisplay-message yes\n%endif\n",
            b"%if 1\n%if 0\ndisplay-message no\n%else\ndisplay-message yes\n%endif\n%endif\n",
        ] {
            let result = parse(input);
            assert_eq!(result.status, CMD_PARSE_SUCCESS, "{:?}", result.error);
            assert!(result.cmdlist.is_some());
        }
    }
}

#[test]
fn command_blocks_and_assignments_exercise_nested_arguments() {
    let _guard = globals();
    unsafe {
        for input in [
            b"if-shell true { display-message yes } { display-message no }".as_slice(),
            b"bind-key x { display-message one ; display-message two }",
            b"FOCUSED=value\ndisplay-message done",
            b"%hidden SECRET=value\ndisplay-message done",
            b"display-message ~",
            b"display-message \"$HOME ${HOME}\"",
        ] {
            let result = parse(input);
            assert_eq!(
                result.status, CMD_PARSE_SUCCESS,
                "{:?}: {:?}",
                input, result.error
            );
        }
    }
}

#[test]
fn parse_state_helpers_cover_scope_and_collection_transitions() {
    let mut state = ParseState::new();
    let token = |s: &CStr| TokenText::from_cstring(s.to_owned());
    let command = ParseCommand::new(1, token(c"display-message"), vec![]);
    assert_eq!(state.start_commands(command).len(), 1);
    assert!(!state.push_scope(&token(c"0")));
    assert!(state.keep_if_active(Vec::new()).is_empty());
    state.invert_scope();
    assert!(state.scope_active());
    assert!(state.replace_scope(&token(c"1")));
    state.pop_scope();
    assert!(state.scope_active());
    let combined = concat(vec![ParseCommand::empty(1)], vec![ParseCommand::empty(2)]);
    assert_eq!(combined.len(), 2);
    let arguments = prepend(
        ParseArgument::String(token(c"one")),
        vec![ParseArgument::String(token(c"two"))],
    );
    assert_eq!(arguments.len(), 2);
    assert_eq!(yylex_is_var(b'A' as i8, 1), 1);
    assert_eq!(yylex_is_var(b'9' as i8, 1), 0);
    assert_eq!(yylex_is_var(b'9' as i8, 0), 1);
    assert_eq!(yylex_is_var(b'-' as i8, 0), 0);
    assert_eq!(yylex_cstring(b"abc\0tail".to_vec()).as_bytes(), b"abc");
}

#[test]
fn interleaved_lexers_keep_input_lines_and_errors_independent() {
    let _guard = globals();
    let first_bytes = Vec::from(b"first\nsecond".as_slice());
    let second_bytes = Vec::from(b"\"\\400\"".as_slice());
    let mut first_input = cmd_parse_input {
        line: 10,
        ..Default::default()
    };
    let mut second_input = cmd_parse_input {
        line: 70,
        ..Default::default()
    };
    {
        let first = std::rc::Rc::new(std::cell::RefCell::new(cmd_parse_state {
            buf: Some(&first_bytes),
            len: first_bytes.len(),
            input: Some(&mut first_input),
            ..Default::default()
        }));
        let second = std::rc::Rc::new(std::cell::RefCell::new(cmd_parse_state {
            buf: Some(&second_bytes),
            len: second_bytes.len(),
            input: Some(&mut second_input),
            ..Default::default()
        }));
        let mut first_tokens = TokenStream(first.clone());
        let mut second_tokens = TokenStream(second.clone());
        assert!(
            matches!(first_tokens.next(), Some(Ok((_, Token::Word(word), _))) if word.as_c_str() == c"first")
        );
        assert!(matches!(second_tokens.next(), Some(Err(_))));
        assert!(second.borrow().error.is_some());
        assert!(matches!(
            first_tokens.next(),
            Some(Ok((_, Token::Newline, _)))
        ));
        assert!(
            matches!(first_tokens.next(), Some(Ok((_, Token::Word(word), _))) if word.as_c_str() == c"second")
        );
        assert_eq!(first.borrow().input().line, 11);
        assert_eq!(second.borrow().input().line, 70);
        assert!(first.borrow().error.is_none());
        assert!(matches!(
            first_tokens.next(),
            Some(Ok((_, Token::Newline, _)))
        ));
        assert!(first_tokens.next().is_none());
    }
    assert_eq!(first_input.line, 11);
    assert_eq!(second_input.line, 70);
}

#[test]
fn lexer_classifies_directives_assignments_and_literal_words() {
    let _guard = globals();
    for (bytes, expected) in [
        (b"NAME=value".as_slice(), "assignment"),
        (b"_=value", "assignment"),
        (b"A1=x=y", "assignment"),
        (b"9NAME=value", "word"),
        (b"A-B=value", "word"),
        (b"=value", "word"),
        (b"''", "word"),
        (b"%1", "word"),
        (b"%%123", "word"),
        (b"%hidden", "hidden"),
        (b"%if", "if"),
        (b"%else", "else"),
        (b"%elif", "elif"),
        (b"%endif", "endif"),
        (b"%unknown", "error"),
    ] {
        let mut input = cmd_parse_input::default();
        let mut lexer = cmd_parse_state {
            buf: Some(bytes),
            len: bytes.len(),
            input: Some(&mut input),
            ..Default::default()
        };
        let actual = match yylex_next(&mut lexer) {
            Ok(Some(Token::Equals(_))) => "assignment",
            Ok(Some(Token::Word(_))) => "word",
            Ok(Some(Token::Hidden)) => "hidden",
            Ok(Some(Token::If)) => "if",
            Ok(Some(Token::Else)) => "else",
            Ok(Some(Token::Elif)) => "elif",
            Ok(Some(Token::Endif)) => "endif",
            Err(_) => "error",
            token => panic!("unexpected token: {token:?}"),
        };
        assert_eq!(actual, expected, "{bytes:?}");
    }
}

#[test]
fn lexer_unicode_escapes_preserve_width_case_and_errors() {
    let _guard = globals();
    for (bytes, expected) in [
        (br"\u0041".as_slice(), Some(c"A")),
        (br"\U00000041", Some(c"A")),
        (br"\u004a", Some(c"J")),
        (br"\u004A", Some(c"J")),
        (br"\u0041B", Some(c"AB")),
        (br"\u00g1", None),
        (br"\u041", None),
        (br"\U0000041", None),
        (br"\UFFFFFFFF", None),
    ] {
        let mut input = cmd_parse_input::default();
        let mut lexer = cmd_parse_state {
            buf: Some(bytes),
            len: bytes.len(),
            input: Some(&mut input),
            ..Default::default()
        };
        match (yylex_next(&mut lexer), expected) {
            (Ok(Some(Token::Word(word))), Some(expected)) => assert_eq!(word.as_c_str(), expected),
            (Err(_), None) => {}
            (actual, _) => panic!("{bytes:?}: unexpected token {actual:?}"),
        }
    }
}

#[test]
fn conditional_formats_use_explicit_targets_and_resolve_missing_targets() {
    use crate::tests::test_fixtures::Target;

    let _guard = globals();
    let mut target = Target::new(17, 4);
    for fs in [target.state(), cmd_find_state::default()] {
        let mut pi = cmd_parse_input {
            flags: CMD_PARSE_PARSEONLY | CMD_PARSE_NOALIAS,
            fs,
            ..Default::default()
        };
        unsafe {
            let result = cmd_parse_from_buffer(
                b"%if #{==:#{pane_width},17}\ndisplay-message matched\n%else\ndisplay-message wrong\n%endif\n",
                Some(&mut pi),
            );
            assert_eq!(result.status, CMD_PARSE_SUCCESS, "{:?}", result.error);
            assert_eq!(
                (result.cmdlist.as_ref().unwrap()).print(0).as_c_str(),
                c"display-message matched"
            );
        }
    }
}
