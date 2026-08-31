use super::*;
use crate::cmd::CmdqItemRef;
use crate::cmd::cmd_display_message::cmd_display_message_entry;
use crate::cmd::cmd_list_print;
use crate::cmd::cmd_parse_from_string;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CmdqStateRef, CmdqType, cmdq_new_state};
use crate::tests::test_fixtures::{globals, seen, zeroed_client};
use ::core::ffi::{CStr, c_char};
use ::std::ffi::CString;

/// The command parser and the format engine read the globals and keep
/// state of their own in more, so the tests that run them take turns.
fn exclusive() -> ::std::sync::MutexGuard<'static, ()> {
    globals()
}

/// A command list parsed from `s`, as the command parser would build it.
/// The list is handed straight to a `Values`, which frees it, so this is
/// not the fixtures' `Args`: that one owns what it parsed.
fn cmdlist(s: &CStr) -> CmdListRef {
    unsafe {
        let mut pr = cmd_parse_from_string(s.as_ptr(), ::core::ptr::null_mut::<cmd_parse_input>());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
        pr.cmdlist.take().unwrap()
    }
}

/// One value in the array the command parser hands to `args_parse`.
enum Value<'a> {
    None,
    String(&'a CStr),
    Commands(&'a CStr),
}

/// An owned value array, freed the way the command parser frees the values
/// it passed in.
struct Values(Vec<args_value_t>);

impl Values {
    fn new(values: &[Value<'_>]) -> Values {
        Values(
            values
                .iter()
                .map(|value| {
                    let mut new = args_value_t::default();
                    match value {
                        Value::None => {}
                        Value::String(s) => {
                            new.value = ArgsValue::String((*s).to_owned());
                        }
                        Value::Commands(s) => {
                            new.value = ArgsValue::Commands {
                                cmdlist: Some(cmdlist(s)),
                                cached: None,
                            };
                        }
                    }
                    new
                })
                .collect(),
        )
    }

    /// The values of a command line: its name first, then its words.
    fn words(words: &[&CStr]) -> Values {
        Values::new(&words.iter().map(|w| Value::String(w)).collect::<Vec<_>>())
    }

    fn ptr(&mut self) -> *mut args_value_t {
        self.0.as_mut_ptr()
    }

    fn count(&self) -> u_int {
        self.0.len() as u_int
    }
}

impl Drop for Values {
    fn drop(&mut self) {
        unsafe { args_free_values(self.0.as_mut_ptr(), self.0.len() as u_int) };
    }
}

/// A template with no callback and no bounds on the argument count.
fn spec(template: &'static CStr) -> args_parse_t {
    args_parse_t {
        template,
        lower: -1,
        upper: -1,
        cb: None,
    }
}

fn spec_cb(cb: args_parse_cb) -> args_parse_t {
    spec_cb_template(c"", cb)
}

fn spec_cb_template(template: &'static CStr, cb: args_parse_cb) -> args_parse_t {
    args_parse_t {
        template,
        lower: -1,
        upper: -1,
        cb,
    }
}

/// The arguments a value array parses to, or the cause it failed with.
fn parse_values(spec: &args_parse_t, values: &mut Values) -> Result<*mut args, String> {
    unsafe {
        let mut cause = None;
        let Some(args) = args_parse(spec, values.ptr(), values.count(), &mut cause) else {
            return Err(cause.unwrap().into_string().unwrap());
        };
        assert!(cause.is_none());
        Ok(Box::into_raw(args))
    }
}

/// The printed form of the arguments a command line parses to, or the
/// cause it failed with.
fn parse(template: &'static CStr, words: &[&CStr]) -> Result<String, String> {
    parse_spec(&spec(template), words)
}

fn parse_spec(spec: &args_parse_t, words: &[&CStr]) -> Result<String, String> {
    let mut values = Values::words(words);
    unsafe {
        let args = parse_values(spec, &mut values)?;
        let printed = args_print(args).to_string_lossy().into_owned();
        args_free(Box::from_raw(args));
        Ok(printed)
    }
}

/// A freshly allocated string value, as `args_set` expects to be given.
fn string_value(s: &CStr) -> Option<Box<args_value_t>> {
    let mut value = Box::new(args_value_t::default());
    value.value = ArgsValue::String(s.to_owned());
    Some(value)
}

#[test]
fn an_empty_command_line_parses_to_empty_arguments() {
    let mut values = Values::words(&[]);
    unsafe {
        let args = parse_values(&spec(c""), &mut values).unwrap();
        assert_eq!(args_count(&*args), 0);
        assert!(args_values(args).is_null());
        assert_eq!(args_print(args).to_string_lossy(), "");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn flags_are_taken_from_the_template() {
    assert_eq!(parse(c"ab", &[c"cmd", c"-a"]), Ok("-a".to_owned()));
    assert_eq!(parse(c"ab", &[c"cmd", c"-ab"]), Ok("-ab".to_owned()));
    assert_eq!(parse(c"ab", &[c"cmd", c"-a", c"-b"]), Ok("-ab".to_owned()));
    assert_eq!(parse(c"ab", &[c"cmd", c"-aa"]), Ok("-aa".to_owned()));
}

#[test]
fn a_word_that_is_not_a_flag_ends_the_flags() {
    assert_eq!(parse(c"a", &[c"cmd", c"x", c"-a"]), Ok("x -a".to_owned()));
    assert_eq!(parse(c"a", &[c"cmd", c"-"]), Ok("-".to_owned()));
    assert_eq!(parse(c"a", &[c"cmd", c"-", c"-a"]), Ok("- -a".to_owned()));
}

#[test]
fn a_command_list_word_ends_the_flags() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::Commands(c"display-message hello"),
    ]);
    unsafe {
        let args = parse_values(&spec_cb(Some(parse_as_either)), &mut values).unwrap();
        assert_eq!(args_count(&*args), 1);
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_double_dash_ends_the_flags_and_is_dropped() {
    assert_eq!(parse(c"a", &[c"cmd", c"--", c"-a"]), Ok("-a".to_owned()));
    assert_eq!(
        parse(c"a", &[c"cmd", c"-a", c"--", c"x"]),
        Ok("-a x".to_owned())
    );
}

#[test]
fn a_question_mark_flag_fails_without_a_cause() {
    let mut values = Values::words(&[c"cmd", c"-?"]);
    let mut cause = None;
    unsafe {
        let args = args_parse(&spec(c"a"), values.ptr(), values.count(), &mut cause);
        assert!(args.is_none());
        assert!(cause.is_none());
    }
}

#[test]
fn a_flag_must_be_alphanumeric_and_in_the_template() {
    assert_eq!(
        parse(c"a", &[c"cmd", c"-."]),
        Err("invalid flag -.".to_owned())
    );
    assert_eq!(
        parse(c"a", &[c"cmd", c"-b"]),
        Err("unknown flag -b".to_owned())
    );
}

#[test]
fn a_flag_argument_is_taken_from_the_rest_of_the_word_or_the_next_one() {
    assert_eq!(parse(c"a:", &[c"cmd", c"-ax"]), Ok("-a x".to_owned()));
    assert_eq!(parse(c"a:", &[c"cmd", c"-a", c"x"]), Ok("-a x".to_owned()));
    assert_eq!(
        parse(c"a:b", &[c"cmd", c"-ba", c"x"]),
        Ok("-b -a x".to_owned())
    );
    assert_eq!(
        parse(c"a:", &[c"cmd", c"-a"]),
        Err("-a expects an argument".to_owned())
    );
}

#[test]
fn a_flag_argument_must_be_a_string() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::String(c"-a"),
        Value::Commands(c"display-message hello"),
    ]);
    {
        assert_eq!(
            parse_values(&spec(c"a:"), &mut values).err(),
            Some("-a argument must be a string".to_owned())
        );
    }
}

#[test]
fn an_optional_flag_argument_may_be_left_out() {
    assert_eq!(parse(c"a::", &[c"cmd", c"-ax"]), Ok("-a x".to_owned()));
    assert_eq!(parse(c"a::", &[c"cmd", c"-a"]), Ok("-a --".to_owned()));
    assert_eq!(
        parse(c"a::b", &[c"cmd", c"-b", c"-a"]),
        Ok("-b -a --".to_owned())
    );
}

#[test]
fn the_argument_count_is_checked_against_the_bounds() {
    let bounded = |lower, upper| args_parse_t {
        template: c"",
        lower,
        upper,
        cb: None,
    };
    assert_eq!(
        parse_spec(&bounded(1, -1), &[c"cmd"]),
        Err("too few arguments (need at least 1)".to_owned())
    );
    assert_eq!(
        parse_spec(&bounded(-1, 1), &[c"cmd", c"x", c"y"]),
        Err("too many arguments (need at most 1)".to_owned())
    );
    assert_eq!(
        parse_spec(&bounded(1, 2), &[c"cmd", c"x"]),
        Ok("x".to_owned())
    );
}

unsafe fn parse_as_string(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_STRING
}

unsafe fn parse_as_commands(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS
}

unsafe fn parse_as_either(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

unsafe fn refuse(_args: &args, idx: u_int, cause: &mut Option<CString>) -> args_parse_type {
    unsafe {
        *cause = Some(xasprintf(c"no argument %u".as_ptr(), fmt_args![idx]));
        ARGS_PARSE_INVALID
    }
}

unsafe fn parse_as_nothing_known(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS + 1
}

#[test]
fn the_callback_says_what_each_argument_must_be() {
    assert_eq!(
        parse_spec(&spec_cb(Some(parse_as_string)), &[c"cmd", c"x"]),
        Ok("x".to_owned())
    );
    assert_eq!(
        parse_spec(&spec_cb(Some(refuse)), &[c"cmd", c"x"]),
        Err("no argument 0".to_owned())
    );
}

#[test]
fn an_argument_that_must_be_a_string_rejects_a_command_list() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::Commands(c"display-message hello"),
    ]);
    {
        assert_eq!(
            parse_values(&spec_cb(Some(parse_as_string)), &mut values).err(),
            Some("argument 1 must be \"string\"".to_owned())
        );
    }
}

#[test]
fn an_argument_that_must_be_a_command_list_rejects_a_string() {
    let mut values = Values::new(&[Value::String(c"cmd"), Value::String(c"x")]);
    {
        assert_eq!(
            parse_values(&spec_cb(Some(parse_as_commands)), &mut values).err(),
            Some("argument 1 must be { commands }".to_owned())
        );
    }
}

#[test]
fn a_command_list_argument_is_kept_and_printed_in_braces() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::Commands(c"display-message hello"),
    ]);
    unsafe {
        let args = parse_values(&spec_cb(Some(parse_as_commands)), &mut values).unwrap();
        assert_eq!(
            args_print(args).to_string_lossy(),
            "{ display-message hello }"
        );
        let value = args_value(args, 0);
        assert!(matches!(
            &(*value).value,
            ArgsValue::Commands { cached: None, .. }
        ));
        assert_eq!(seen(args_string(&*args, 0)), "display-message hello");
        assert!(matches!(
            &(*value).value,
            ArgsValue::Commands {
                cached: Some(_),
                ..
            }
        ));
        args_free(Box::from_raw(args));
    }
}

#[test]
fn either_kind_of_argument_is_copied_as_it_comes() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::String(c"x"),
        Value::Commands(c"display-message hello"),
        Value::None,
    ]);
    unsafe {
        let args = parse_values(&spec_cb(Some(parse_as_either)), &mut values).unwrap();
        assert_eq!(args_count(&*args), 3);
        assert_eq!(seen(args_string(&*args, 0)), "x");
        assert_eq!(seen(args_string(&*args, 1)), "display-message hello");
        assert_eq!(seen(args_string(&*args, 2)), "");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn an_argument_of_an_unknown_kind_is_counted_but_left_empty() {
    let mut values = Values::new(&[Value::String(c"cmd"), Value::String(c"x")]);
    unsafe {
        let args = parse_values(&spec_cb(Some(parse_as_nothing_known)), &mut values).unwrap();
        assert_eq!(args_count(&*args), 1);
        assert!(matches!(&(*args_value(args, 0)).value, ArgsValue::None));
        assert_eq!(seen(args_string(&*args, 0)), "");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn the_type_names_are_what_the_log_prints() {
    assert_eq!(args_value_type_to_string(&ArgsValue::None), c"NONE");
    assert_eq!(
        args_value_type_to_string(&ArgsValue::String(CString::default())),
        c"STRING"
    );
    assert_eq!(
        args_value_type_to_string(&ArgsValue::Commands {
            cmdlist: None,
            cached: None,
        }),
        c"COMMANDS"
    );
}

#[test]
fn a_flag_is_counted_every_time_it_is_given() {
    unsafe {
        let args = Box::into_raw(args_create());
        assert_eq!(args_has(&*args, b'a'), 0);
        args_set(args, b'a', None, 0);
        assert_eq!(args_has(&*args, b'a'), 1);
        args_set(args, b'a', None, 0);
        assert_eq!(args_has(&*args, b'a'), 2);
        assert_eq!(args_has(&*args, b'b'), 0);
        args_free(Box::from_raw(args));
    }
}

#[test]
fn the_last_value_given_for_a_flag_is_the_one_read_back() {
    unsafe {
        let args = Box::into_raw(args_create());
        assert!(args_get(&*args, b'a').is_null());
        args_set(args, b'a', None, 0);
        assert!(args_get(&*args, b'a').is_null());
        args_set(args, b'a', string_value(c"one"), 0);
        args_set(args, b'a', string_value(c"two"), 0);
        assert_eq!(seen(args_get(&*args, b'a')), "two");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn the_values_of_a_flag_come_back_in_order() {
    unsafe {
        let args = Box::into_raw(args_create());
        assert!(args_value_list(&*args, b'a').is_empty());
        args_set(args, b'a', string_value(c"one"), 0);
        args_set(args, b'a', string_value(c"two"), 0);
        let values = args_value_list(&*args, b'a');
        assert_eq!(values.len(), 2);
        assert_eq!((*values[0]).value.string(), c"one");
        assert_eq!((*values[1]).value.string(), c"two");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_value_of_no_type_is_thrown_away_by_args_set() {
    unsafe {
        let args = Box::into_raw(args_create());
        let value = Box::new(args_value_t::default());
        args_set(args, b'a', Some(value), 0);
        assert_eq!(args_has(&*args, b'a'), 1);
        assert!(args_value_list(&*args, b'a').is_empty());
        args_free(Box::from_raw(args));
    }
}

#[test]
fn the_flags_are_walked_in_order() {
    unsafe {
        let args = Box::into_raw(args_create());
        assert_eq!(args_flags(&*args).next(), None);
        for flag in *b"cab" {
            args_set(args, flag, None, 0);
        }
        let walked: Vec<u_char> = args_flags(&*args).collect();
        assert_eq!(walked, [b'a', b'b', b'c']);
        args_free(Box::from_raw(args));
    }
}

#[test]
fn the_arguments_are_read_by_index() {
    let mut values = Values::words(&[c"cmd", c"x", c"y"]);
    unsafe {
        let args = parse_values(&spec(c""), &mut values).unwrap();
        assert_eq!(args_count(&*args), 2);
        assert_eq!(seen(args_string(&*args, 0)), "x");
        assert_eq!(seen(args_string(&*args, 1)), "y");
        assert!(args_string(&*args, 2).is_null());
        assert_eq!(args_value(args, 0), args_values(args));
        assert!(args_value(args, 2).is_null());
        args_free(Box::from_raw(args));
    }
}

#[test]
fn arguments_are_printed_back_as_a_command_line() {
    assert_eq!(
        parse(c"ab:c::", &[c"cmd", c"-a", c"-b", c"one", c"-c", c"x"]),
        Ok("-a -b one -c x".to_owned())
    );
    assert_eq!(
        parse(c"ab:", &[c"cmd", c"-aab", c"one", c"x", c"y"]),
        Ok("-aa -b one x y".to_owned())
    );
}

#[test]
fn printing_quotes_what_would_otherwise_be_read_back_wrongly() {
    assert_eq!(
        parse(c"", &[c"cmd", c"a b", c"", c"a#b"]),
        Ok("\"a b\" '' \"a#b\"".to_owned())
    );
}

#[test]
fn escaping_quotes_a_string_the_way_the_parser_would_read_it_back() {
    let escape = |s: &CStr| unsafe { args_escape(s.as_ptr()).to_string_lossy().into_owned() };
    assert_eq!(escape(c""), "''");
    assert_eq!(escape(c"a"), "a");
    assert_eq!(escape(c"abc"), "abc");
    assert_eq!(escape(c" "), "\" \"");
    assert_eq!(escape(c"~"), "\\~");
    assert_eq!(escape(c"#"), "\\#");
    assert_eq!(escape(c"\""), "\\\"");
    assert_eq!(escape(c"a b"), "\"a b\"");
    assert_eq!(escape(c"a#b"), "\"a#b\"");
    assert_eq!(escape(c"a\"b"), "'a\"b'");
    assert_eq!(escape(c"~a"), "\\~a");
    assert_eq!(escape(c"~ a"), "\"\\~ a\"");
    assert_eq!(escape(c"~'a"), "\"\\~'a\"");
    assert_eq!(escape(c"a\tb"), "a\\tb");
    assert_eq!(escape(c"a\nb"), "a\\nb");
    assert_eq!(escape(c"a\x07b"), "a\\ab");
}

#[test]
fn arguments_are_copied_with_the_template_words_replaced() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::String(c"-a"),
        Value::String(c"%1-%2"),
        Value::String(c"x %1"),
        Value::Commands(c"display-message hello"),
        Value::None,
    ]);
    unsafe {
        let args =
            parse_values(&spec_cb_template(c"a", Some(parse_as_either)), &mut values).unwrap();
        let argv = vec![CString::new("one").unwrap(), CString::new("two").unwrap()];
        let mut copy = args_copy(args, &argv);
        let copy_ptr = &raw mut *copy;
        assert_eq!(
            args_print(copy_ptr).to_string_lossy(),
            "-a one-two \"x one\" { display-message hello } "
        );
        drop(copy);
        args_free(Box::from_raw(args));
    }
}

#[test]
fn copying_keeps_the_flags_and_their_values() {
    unsafe {
        let args = Box::into_raw(args_create());
        args_set(args, b'a', None, 0);
        args_set(args, b'a', None, 0);
        args_set(args, b'b', string_value(c"one"), 0);
        let mut copy = args_copy(args, &[]);
        let copy_ptr = &raw mut *copy;
        assert_eq!(args_has(&*copy_ptr, b'a'), 2);
        assert_eq!(seen(args_get(&*copy_ptr, b'b')), "one");
        assert_eq!(args_print(copy_ptr).to_string_lossy(), "-aa -b one");
        drop(copy);
        args_free(Box::from_raw(args));
    }
}

#[test]
fn an_optional_value_flag_is_printed_before_the_arguments() {
    unsafe {
        let args = Box::into_raw(args_create());
        args_set(args, b'a', None, ARGS_ENTRY_OPTIONAL_VALUE);
        args_set(args, b'b', string_value(c"one"), 0);
        assert_eq!(args_print(args).to_string_lossy(), "-a -b one");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_value_array_is_built_from_a_word_list() {
    let argv = vec![CString::new("one").unwrap(), CString::new("two").unwrap()];
    let values = args_from_vector(&argv);
    assert_eq!(values.len(), 2);
    assert!(matches!(&values[0].value, ArgsValue::String(s) if s.as_bytes() == b"one"));
    assert!(matches!(&values[1].value, ArgsValue::String(s) if s.as_bytes() == b"two"));
}

#[test]
fn the_arguments_flatten_back_to_a_word_list() {
    let _guard = exclusive();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::String(c"x"),
        Value::Commands(c"display-message hello"),
        Value::None,
    ]);
    unsafe {
        let args = parse_values(&spec_cb(Some(parse_as_either)), &mut values).unwrap();
        let argv = args_to_vector(&*args);
        assert_eq!(argv.len(), 2);
        assert_eq!(argv[0].as_bytes(), b"x");
        assert_eq!(argv[1].as_bytes(), b"display-message hello");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_number_argument_is_read_from_the_last_value_of_a_flag() {
    unsafe {
        let args = Box::into_raw(args_create());
        let mut cause = None;
        assert_eq!(args_strtonum(&*args, b'a', 0, 10, &mut cause), 0);
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");

        args_set(args, b'a', None, 0);
        assert_eq!(args_strtonum(&*args, b'a', 0, 10, &mut cause), 0);
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");

        args_set(args, b'a', string_value(c"3"), 0);
        assert_eq!(args_strtonum(&*args, b'a', 0, 10, &mut cause), 3);
        assert!(cause.is_none());

        args_set(args, b'a', string_value(c"11"), 0);
        assert_eq!(args_strtonum(&*args, b'a', 0, 10, &mut cause), 0);
        assert_eq!(cause.take().unwrap().to_string_lossy(), "too large");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_number_argument_has_to_be_a_string() {
    let _guard = exclusive();
    unsafe {
        let args = Box::into_raw(args_create());
        let mut value = Box::new(args_value_t::default());
        value.value = ArgsValue::Commands {
            cmdlist: Some(cmdlist(c"display-message hello")),
            cached: None,
        };
        args_set(args, b'a', Some(value), 0);
        let mut cause = None;
        assert_eq!(args_strtonum(&*args, b'a', 0, 10, &mut cause), 0);
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_number_argument_can_be_expanded_first() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    unsafe {
        let args = Box::into_raw(args_create());
        let mut cause = None;
        assert_eq!(
            args_strtonum_and_expand(&*args, b'a', 0, 10, runner.item(), &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");

        args_set(args, b'a', None, 0);
        assert_eq!(
            args_strtonum_and_expand(&*args, b'a', 0, 10, runner.item(), &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");

        args_set(args, b'a', string_value(c"#{e|+|:1,2}"), 0);
        assert_eq!(
            args_strtonum_and_expand(&*args, b'a', 0, 10, runner.item(), &mut cause),
            3
        );
        assert!(cause.is_none());

        args_set(args, b'a', string_value(c"x"), 0);
        assert_eq!(
            args_strtonum_and_expand(&*args, b'a', 0, 10, runner.item(), &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "invalid");
        args_free(Box::from_raw(args));
    }
}

#[test]
fn a_percentage_is_taken_of_the_current_value() {
    let percentage = |value: &CStr| unsafe {
        let mut cause = None;
        let ll = args_string_percentage(value.as_ptr(), 1, 100, 50, &mut cause);
        if cause.is_none() {
            Ok(ll)
        } else {
            Err(cause.take().unwrap().to_string_lossy().into_owned())
        }
    };
    assert_eq!(percentage(c""), Err("empty".to_owned()));
    assert_eq!(percentage(c"10"), Ok(10));
    assert_eq!(percentage(c"x"), Err("invalid".to_owned()));
    assert_eq!(percentage(c"50%"), Ok(25));
    assert_eq!(percentage(c"x%"), Err("invalid".to_owned()));
    assert_eq!(percentage(c"1%"), Err("too small".to_owned()));
    assert_eq!(percentage(c"101%"), Err("too large".to_owned()));
    unsafe {
        let mut cause = None;
        assert_eq!(
            args_string_percentage(c"100%".as_ptr(), 0, 10, 50, &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "too large");
    }
}

#[test]
fn a_percentage_argument_comes_from_the_last_value_of_a_flag() {
    unsafe {
        let args = Box::into_raw(args_create());
        let mut cause = None;
        assert_eq!(args_percentage(&*args, b'a', 0, 100, 50, &mut cause), 0);
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");

        args_set(args, b'a', None, 0);
        assert_eq!(args_percentage(&*args, b'a', 0, 100, 50, &mut cause), 0);
        assert_eq!(cause.take().unwrap().to_string_lossy(), "empty");

        args_set(args, b'a', string_value(c"50%"), 0);
        assert_eq!(args_percentage(&*args, b'a', 0, 100, 50, &mut cause), 25);
        assert!(cause.is_none());
        args_free(Box::from_raw(args));
    }
}

#[test]
fn an_expanded_percentage_argument_comes_from_the_last_value_of_a_flag() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    unsafe {
        let args = Box::into_raw(args_create());
        let mut cause = None;
        assert_eq!(
            args_percentage_and_expand(&*args, b'a', 0, 100, 50, runner.item(), &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "missing");

        args_set(args, b'a', None, 0);
        assert_eq!(
            args_percentage_and_expand(&*args, b'a', 0, 100, 50, runner.item(), &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "empty");

        args_set(args, b'a', string_value(c"#{e|+|:20,30}%"), 0);
        assert_eq!(
            args_percentage_and_expand(&*args, b'a', 0, 100, 50, runner.item(), &mut cause),
            25
        );
        assert!(cause.is_none());
        args_free(Box::from_raw(args));
    }
}

#[test]
fn an_expanded_percentage_is_taken_of_the_current_value() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let item = runner.item();
    let percentage = |value: &CStr| unsafe {
        let mut cause = None;
        let ll = args_string_percentage_and_expand(value.as_ptr(), 1, 100, 50, item, &mut cause);
        if cause.is_none() {
            Ok(ll)
        } else {
            Err(cause.take().unwrap().to_string_lossy().into_owned())
        }
    };
    assert_eq!(percentage(c""), Err("invalid".to_owned()));
    assert_eq!(percentage(c"10"), Ok(10));
    assert_eq!(percentage(c"x"), Err("invalid".to_owned()));
    assert_eq!(percentage(c"#{e|+|:20,30}%"), Ok(25));
    assert_eq!(percentage(c"x%"), Err("invalid".to_owned()));
    assert_eq!(percentage(c"1%"), Err("too small".to_owned()));
    assert_eq!(percentage(c"101%"), Err("too large".to_owned()));
    unsafe {
        let mut cause = None;
        assert_eq!(
            args_string_percentage_and_expand(c"100%".as_ptr(), 0, 10, 50, item, &mut cause),
            0
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "too large");
    }
}

/// The command and queue item a command runs under, with the fields this
/// module reads: the item's client, command and state, and the command's
/// arguments and source file.
struct Runner {
    item: CmdqItemRef,
    _state: CmdqStateRef,
    cmdlist: CmdListRef,
    client: ClientRef,
}

impl Runner {
    fn new() -> Runner {
        unsafe {
            let state = cmdq_new_state(::core::ptr::null_mut(), ::core::ptr::null_mut(), 0);
            let mut cmd = crate::tests::test_fixtures::empty_cmd();
            cmd.entry = &cmd_display_message_entry;
            cmd.file = Some(c"args.conf".to_owned());
            cmd.line = 7;
            let cmdlist = crate::cmd::cmd_list_new();
            crate::cmd::cmd_list_append(&cmdlist, cmd);
            let client = zeroed_client();
            let item = crate::tests::test_fixtures::zeroed_cmdq_item(state.clone());
            item.item().type_0 = CmdqType::Command {
                cmdlist: Some(cmdlist.clone()),
                at: 0,
            };
            Runner {
                item,
                _state: state,
                cmdlist,
                client,
            }
        }
    }

    fn item(&mut self) -> *mut cmdq_item {
        self.item.as_ptr()
    }

    fn cmd(&mut self) -> *mut cmd {
        unsafe { crate::cmd::cmd_list_at(&self.cmdlist, 0) }
    }

    /// Run the command with the arguments a command line parses to.
    fn with_args(&mut self, values: &mut Values) -> *mut args {
        {
            let args = parse_values(&spec_cb(Some(parse_as_either)), values).unwrap();
            unsafe {
                (*self.cmd()).args = Some(Box::from_raw(args));
                (*self.cmd()).args_ptr()
            }
        }
    }
}

#[test]
fn a_command_list_argument_is_prepared_as_it_is() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::new(&[
        Value::String(c"cmd"),
        Value::Commands(c"display-message hello"),
    ]);
    unsafe {
        runner.with_args(&mut values);
        let mut state = args_make_commands_prepare(
            &*runner.cmd(),
            runner.item(),
            0,
            ::core::ptr::null::<c_char>(),
            0,
            0,
        );
        assert_eq!(
            args_make_commands_get_command(&state).as_c_str(),
            c"display-message"
        );
        let mut error = None;
        let same = args_make_commands(&mut state, &[], &mut error);
        assert_eq!(
            cmd_list_print(same.as_ref().unwrap(), 0).to_string_lossy(),
            "display-message hello"
        );

        let argv = vec![CString::new("one").unwrap()];
        let copy = args_make_commands(&mut state, &argv, &mut error);
        assert_ne!(copy, same);
        assert_eq!(
            cmd_list_print(copy.as_ref().unwrap(), 0).to_string_lossy(),
            "display-message hello"
        );
        drop(copy);
    }
}

#[test]
fn an_empty_command_list_argument_has_no_command_name() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::new(&[Value::String(c"cmd"), Value::Commands(c"")]);
    unsafe {
        runner.with_args(&mut values);
        let state = args_make_commands_prepare(
            &*runner.cmd(),
            runner.item(),
            0,
            ::core::ptr::null::<c_char>(),
            0,
            0,
        );
        assert_eq!(args_make_commands_get_command(&state).as_c_str(), c"");
    }
}

#[test]
fn a_string_argument_is_parsed_when_the_commands_are_made() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::words(&[c"cmd", c"display-message %1"]);
    unsafe {
        runner.with_args(&mut values);
        let mut state = args_make_commands_prepare(
            &*runner.cmd(),
            runner.item(),
            0,
            ::core::ptr::null::<c_char>(),
            0,
            0,
        );
        assert_eq!(
            args_make_commands_get_command(&state).as_c_str(),
            c"display-message"
        );
        let mut error = None;
        let argv = vec![CString::new("hello").unwrap()];
        let cmdlist = args_make_commands(&mut state, &argv, &mut error);
        assert_eq!(
            cmd_list_print(cmdlist.as_ref().unwrap(), 0).to_string_lossy(),
            "display-message hello"
        );
        drop(cmdlist);
    }
}

#[test]
fn a_string_argument_that_is_not_a_command_gives_the_parse_error() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::words(&[c"cmd", c"nosuchcommand"]);
    unsafe {
        runner.with_args(&mut values);
        let mut state = args_make_commands_prepare(
            &*runner.cmd(),
            runner.item(),
            0,
            ::core::ptr::null::<c_char>(),
            0,
            0,
        );
        let mut error = None;
        let cmdlist = args_make_commands(&mut state, &[], &mut error);
        assert!(cmdlist.is_none());
        assert_eq!(
            error.take().unwrap().to_string_lossy(),
            "args.conf:7: unknown command: nosuchcommand"
        );
    }
}

#[test]
fn a_default_command_stands_in_for_a_missing_argument() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::words(&[c"cmd"]);
    unsafe {
        runner.with_args(&mut values);
        let state = args_make_commands_prepare(
            &*runner.cmd(),
            runner.item(),
            0,
            c"display-message #{e|+|:1,2}".as_ptr(),
            0,
            0,
        );
        assert_eq!(
            args_make_commands_get_command(&state).as_c_str(),
            c"display-message"
        );
    }
}

#[test]
fn a_prepared_command_can_be_expanded_first() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::words(&[c"cmd"]);
    unsafe {
        cmdq_set_target_client(runner.item.as_ptr(), &raw mut *runner.client);
        runner.with_args(&mut values);
        let state = args_make_commands_prepare(
            &*runner.cmd(),
            runner.item(),
            0,
            c"display-message #{e|+|:1,2}".as_ptr(),
            1,
            1,
        );
        assert_eq!(state.cmd.as_deref(), Some(c"display-message 3"));
        assert_eq!(state.pi.item, runner.item());
        assert_eq!(state.pi.file(), Some(c"args.conf"));
        assert_eq!(state.pi.line, 7);
    }
}

#[test]
fn making_the_commands_at_once_reports_a_parse_error_to_the_queue() {
    let _guard = exclusive();
    let mut runner = Runner::new();
    let mut values = Values::words(&[c"cmd", c"display-message hello", c"nosuchcommand"]);
    unsafe {
        runner.with_args(&mut values);
        let cmdlist = args_make_commands_now(&*runner.cmd(), runner.item(), 0, 0);
        assert!(cmdlist.is_some());
        assert_eq!(
            cmd_list_print(cmdlist.as_ref().unwrap(), 0).to_string_lossy(),
            "display-message hello"
        );
        drop(cmdlist);

        assert!(args_make_commands_now(&*runner.cmd(), runner.item(), 1, 0).is_none());
    }
}
