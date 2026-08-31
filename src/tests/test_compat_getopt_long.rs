use super::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;
use ::std::sync::{Mutex, MutexGuard};

/// A turn at the parser's own globals — the index into the argument list,
/// the argument last read and the place it keeps inside the current
/// argument — which cargo's parallel threads would otherwise share. Taking
/// the turn also asks for a fresh start: a zero index is what the parser
/// reads as one.
fn parser() -> MutexGuard<'static, ()> {
    static PARSER: Mutex<()> = Mutex::new(());
    let guard = PARSER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe { BSDopterr = 0 };
    reset();
    guard
}

/// Asks the parser for a fresh start: a zero index is what it reads as
/// one, and the place inside the current argument is given up so that no
/// run reads what the run before it left behind.
fn reset() {
    unsafe {
        BSDoptind = 0;
        BSDoptopt = '?' as c_int;
        BSDoptarg = null_mut::<c_char>();
        place = EMSG;
    }
}

/// An argument list as `main` is handed one: the strings, and the
/// null-terminated array of pointers to them that the parser may reorder
/// in place. The terminator matters — a long option whose argument is
/// missing reads one past the end and expects to find it.
struct Argv {
    strings: Vec<CString>,
    ptrs: Vec<*mut c_char>,
}

impl Argv {
    fn new(args: &[&str]) -> Argv {
        let strings: Vec<CString> = args
            .iter()
            .map(|a| CString::new(*a).expect("an argument has no NUL"))
            .collect();
        let mut ptrs: Vec<*mut c_char> =
            strings.iter().map(|s| s.as_ptr() as *mut c_char).collect();
        ptrs.push(null_mut::<c_char>());
        Argv { strings, ptrs }
    }

    fn slice(&mut self) -> &mut [*mut c_char] {
        &mut self.ptrs
    }

    /// What the list holds now, which permutation may have reordered.
    fn now(&self) -> Vec<String> {
        self.ptrs[..self.ptrs.len() - 1]
            .iter()
            .map(|p| unsafe { CStr::from_ptr(*p) }.to_string_lossy().into_owned())
            .collect()
    }
}

/// What one run of the parser answered: every option in turn with the
/// argument it took, where it left the index into the argument list, the
/// option it last complained about, the long-option index it last
/// reported, and the argument list as it stands at the end.
#[derive(Debug, PartialEq, Eq)]
struct Run {
    opts: Vec<(char, Option<String>)>,
    optind: c_int,
    optopt: c_int,
    idx: c_int,
    argv: Vec<String>,
}

/// Runs the parser over `args` until it says there is nothing left. An
/// empty `long` stands for no long options at all.
fn drive(args: &[&str], options: &CStr, long: &[option_t], flags: c_int) -> Run {
    reset();
    let mut argv = Argv::new(args);
    let long_ptr = if long.is_empty() {
        null::<option_t>()
    } else {
        long.as_ptr()
    };
    let mut idx: c_int = -1;
    let mut opts = Vec::new();
    loop {
        let ret = unsafe {
            getopt_internal(
                argv.slice(),
                options.as_ptr(),
                long_ptr,
                &raw mut idx,
                flags,
            )
        };
        if ret == -1 {
            break;
        }
        let arg = unsafe {
            if BSDoptarg.is_null() {
                None
            } else {
                Some(CStr::from_ptr(BSDoptarg).to_string_lossy().into_owned())
            }
        };
        opts.push((
            char::from_u32(ret as u32).expect("an option is a character"),
            arg,
        ));
        assert!(opts.len() < 32, "the parser is not making progress");
    }
    unsafe {
        Run {
            opts,
            optind: BSDoptind,
            optopt: BSDoptopt,
            idx,
            argv: argv.now(),
        }
    }
}

/// The same through the public entry, which takes no long options and no
/// flags.
fn getopt(args: &[&str], options: &CStr) -> Run {
    reset();
    let mut argv = Argv::new(args);
    let mut opts = Vec::new();
    loop {
        let ret = unsafe { BSDgetopt(argv.slice(), options.as_ptr()) };
        if ret == -1 {
            break;
        }
        let arg = unsafe {
            if BSDoptarg.is_null() {
                None
            } else {
                Some(CStr::from_ptr(BSDoptarg).to_string_lossy().into_owned())
            }
        };
        opts.push((
            char::from_u32(ret as u32).expect("an option is a character"),
            arg,
        ));
        assert!(opts.len() < 32, "the parser is not making progress");
    }
    unsafe {
        Run {
            opts,
            optind: BSDoptind,
            optopt: BSDoptopt,
            idx: -1,
            argv: argv.now(),
        }
    }
}

/// One long option, as a caller would declare it.
fn long(name: &'static CStr, has_arg: c_int, val: c_int) -> option_t {
    option_t {
        name: name.as_ptr(),
        has_arg,
        flag: null_mut::<c_int>(),
        val,
    }
}

/// The end of a long-option table.
fn end() -> option_t {
    option_t {
        name: null::<c_char>(),
        has_arg: 0,
        flag: null_mut::<c_int>(),
        val: 0,
    }
}

fn opts(run: &Run) -> Vec<(char, Option<&str>)> {
    run.opts.iter().map(|(c, a)| (*c, a.as_deref())).collect()
}

#[test]
fn options_and_their_arguments_are_read() {
    let _guard = parser();
    let run = getopt(&["tmux", "-2", "-f", "conf", "-Lname", "rest"], c"2f:L:");
    assert_eq!(
        opts(&run),
        [('2', None), ('f', Some("conf")), ('L', Some("name"))]
    );
    assert_eq!(run.optind, 5);
}

#[test]
fn options_written_together_are_read_one_by_one() {
    let _guard = parser();
    let run = getopt(&["tmux", "-2Cf", "conf"], c"2Cf:");
    assert_eq!(opts(&run), [('2', None), ('C', None), ('f', Some("conf"))]);
    assert_eq!(run.optind, 3);
}

#[test]
fn a_double_dash_ends_the_options() {
    let _guard = parser();
    let run = getopt(&["tmux", "-2", "--", "-f"], c"2f:");
    assert_eq!(opts(&run), [('2', None)]);
    assert_eq!(run.optind, 3);
}

#[test]
fn the_first_plain_argument_ends_the_options() {
    let _guard = parser();
    let run = getopt(&["tmux", "-2", "rest", "-f", "x"], c"2f:");
    assert_eq!(opts(&run), [('2', None)]);
    assert_eq!(run.optind, 2);
}

#[test]
fn a_lone_dash_is_a_plain_argument() {
    let _guard = parser();
    let run = getopt(&["tmux", "-", "-2"], c"2");
    assert_eq!(opts(&run), []);
    assert_eq!(run.optind, 1);
}

/// Unless the option string names it, in which case it is an option like
/// any other.
#[test]
fn a_lone_dash_is_an_option_when_the_string_names_it() {
    let _guard = parser();
    let run = getopt(&["tmux", "-", "-2"], c"2-");
    assert_eq!(opts(&run), [('-', None), ('2', None)]);
    assert_eq!(run.optind, 3);
}

/// A dash written after an option, where no option is named `-`, ends the
/// parse without complaint rather than being read as an unknown option.
#[test]
fn a_dash_at_the_end_of_a_group_ends_the_parse() {
    let _guard = parser();
    let run = getopt(&["tmux", "-2-", "x"], c"2");
    assert_eq!(opts(&run), [('2', None)]);
    assert_eq!(run.optind, 1);
}

#[test]
fn an_unknown_option_is_reported() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let run = getopt(&["tmux", "-x", "-2"], c"2");
    assert_eq!(opts(&run), [('?', None), ('2', None)]);
    assert_eq!(run.optind, 3);
}

#[test]
fn a_colon_is_never_an_option() {
    let _guard = parser();
    let run = getopt(&["tmux", "-:"], c":2");
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, ':' as c_int);
}

#[test]
fn an_option_whose_argument_is_missing_is_reported() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let run = getopt(&["tmux", "-f"], c"f:");
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 'f' as c_int);
    assert_eq!(run.optind, 2);
}

/// An option string that starts with a colon asks for the missing
/// argument to be reported as a colon rather than a question mark, and
/// silences the parser's own complaints.
#[test]
fn a_leading_colon_asks_for_quiet_reporting() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let run = getopt(&["tmux", "-f"], c":f:");
    assert_eq!(opts(&run), [(':', None)]);
    assert_eq!(run.optopt, 'f' as c_int);
}

#[test]
fn an_optional_argument_is_only_taken_from_the_same_word() {
    let _guard = parser();
    assert_eq!(
        opts(&getopt(&["tmux", "-ox", "y"], c"o::")),
        [('o', Some("x"))]
    );
    let run = getopt(&["tmux", "-o", "y"], c"o::");
    assert_eq!(opts(&run), [('o', None)]);
    assert_eq!(run.optind, 2);
}

/// An option string that starts with a dash hands every plain argument
/// back as option 1, in the order they were written.
#[test]
fn a_leading_dash_hands_back_every_argument() {
    let _guard = parser();
    let run = drive(&["tmux", "x", "-2", "y"], c"-2", &[], 0);
    assert_eq!(
        opts(&run),
        [
            (1_u8 as char, Some("x")),
            ('2', None),
            (1_u8 as char, Some("y"))
        ]
    );
    assert_eq!(run.optind, 4);
}

/// An option string that starts with a plus asks for the parse to stop at
/// the first plain argument, which is what it does anyway here.
#[test]
fn a_leading_plus_stops_at_the_first_argument() {
    let _guard = parser();
    let run = drive(&["tmux", "-2", "x", "-f", "y"], c"+2f:", &[], 0);
    assert_eq!(opts(&run), [('2', None)]);
    assert_eq!(run.optind, 2);
}

#[test]
fn a_null_option_string_reads_nothing() {
    let _guard = parser();
    reset();
    let mut argv = Argv::new(&["tmux", "-2"]);
    let answer = unsafe {
        getopt_internal(
            argv.slice(),
            null::<c_char>(),
            null::<option_t>(),
            null_mut::<c_int>(),
            0,
        )
    };
    assert_eq!(answer, -1);
}

#[test]
fn arguments_are_moved_behind_the_options_when_asked() {
    let _guard = parser();
    let run = drive(
        &["tmux", "x", "-2", "y", "-f", "conf", "z"],
        c"2f:",
        &[],
        FLAG_PERMUTE,
    );
    assert_eq!(opts(&run), [('2', None), ('f', Some("conf"))]);
    assert_eq!(run.argv, ["tmux", "-2", "-f", "conf", "x", "y", "z"]);
    assert_eq!(run.optind, 4);
}

#[test]
fn a_double_dash_stops_the_moving_too() {
    let _guard = parser();
    let run = drive(&["tmux", "x", "-2", "--", "-f"], c"2f:", &[], FLAG_PERMUTE);
    assert_eq!(opts(&run), [('2', None)]);
    assert_eq!(run.argv, ["tmux", "-2", "--", "x", "-f"]);
    assert_eq!(run.optind, 3);
}

/// The last move happens when the arguments run out, not only when the
/// next option turns up.
#[test]
fn arguments_are_moved_when_the_list_runs_out() {
    let _guard = parser();
    let run = drive(&["tmux", "x", "-2"], c"2", &[], FLAG_PERMUTE);
    assert_eq!(opts(&run), [('2', None)]);
    assert_eq!(run.argv, ["tmux", "-2", "x"]);
    assert_eq!(run.optind, 2);
}

#[test]
fn arguments_alone_are_left_where_they_are() {
    let _guard = parser();
    let run = drive(&["tmux", "x", "y"], c"2", &[], FLAG_PERMUTE);
    assert_eq!(opts(&run), []);
    assert_eq!(run.argv, ["tmux", "x", "y"]);
    assert_eq!(run.optind, 1);
}

#[test]
fn a_long_option_is_read_by_its_whole_name() {
    let _guard = parser();
    let table = [long(c"file", required_argument, 'f' as c_int), end()];
    let run = drive(&["tmux", "--file", "conf"], c"f:", &table, 0);
    assert_eq!(opts(&run), [('f', Some("conf"))]);
    assert_eq!(run.idx, 0);
    assert_eq!(run.optind, 3);
}

#[test]
fn a_long_option_takes_its_argument_after_an_equals_sign() {
    let _guard = parser();
    let table = [
        long(c"file", required_argument, 'f' as c_int),
        long(c"tag", optional_argument, 't' as c_int),
        end(),
    ];
    assert_eq!(
        opts(&drive(&["tmux", "--file=conf"], c"f:", &table, 0)),
        [('f', Some("conf"))]
    );
    assert_eq!(
        opts(&drive(&["tmux", "--tag=x"], c"f:", &table, 0)),
        [('t', Some("x"))]
    );
    assert_eq!(
        opts(&drive(&["tmux", "--tag"], c"f:", &table, 0)),
        [('t', None)]
    );
}

#[test]
fn a_long_option_may_be_shortened_while_it_stays_the_only_one() {
    let _guard = parser();
    let table = [
        long(c"file", no_argument, 'f' as c_int),
        long(c"quiet", no_argument, 'q' as c_int),
        end(),
    ];
    assert_eq!(
        opts(&drive(&["tmux", "--fi"], c"fq", &table, 0)),
        [('f', None)]
    );
}

#[test]
fn a_shortened_long_option_that_fits_two_is_refused() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let table = [
        long(c"file", no_argument, 'f' as c_int),
        long(c"filter", no_argument, 'F' as c_int),
        end(),
    ];
    let run = drive(&["tmux", "--fil"], c"fF", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 0);
}

#[test]
fn a_long_option_that_takes_no_argument_refuses_one() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let table = [long(c"quiet", no_argument, 'q' as c_int), end()];
    let run = drive(&["tmux", "--quiet=x"], c"q", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 'q' as c_int);
    let run = drive(&["tmux", "--quiet=x"], c":q", &table, 0);
    assert_eq!(opts(&run), [(':', None)]);
}

#[test]
fn a_long_option_whose_argument_is_missing_is_reported() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let table = [long(c"file", required_argument, 'f' as c_int), end()];
    let run = drive(&["tmux", "--file"], c"f:", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 'f' as c_int);
    let run = drive(&["tmux", "--file"], c":f:", &table, 0);
    assert_eq!(opts(&run), [(':', None)]);
}

#[test]
fn a_long_option_nobody_declared_is_reported() {
    let _guard = parser();
    unsafe { BSDopterr = 1 };
    let table = [long(c"file", no_argument, 'f' as c_int), end()];
    let run = drive(&["tmux", "--zzz"], c"f", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 0);
}

/// A long option carrying a flag writes its value there and answers zero
/// rather than the value itself.
#[test]
fn a_long_option_may_write_its_value_into_a_flag() {
    let _guard = parser();
    let mut flag: c_int = 0;
    let table = [
        option_t {
            name: c"quiet".as_ptr(),
            has_arg: no_argument,
            flag: &raw mut flag,
            val: 7,
        },
        end(),
    ];
    let run = drive(&["tmux", "--quiet"], c"q", &table, 0);
    assert_eq!(opts(&run), [('\0', None)]);
    assert_eq!(flag, 7);
    let table = [
        option_t {
            name: c"quiet".as_ptr(),
            has_arg: no_argument,
            flag: &raw mut flag,
            val: 7,
        },
        end(),
    ];
    let run = drive(&["tmux", "--quiet=x"], c"q", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 0);
    let table = [
        option_t {
            name: c"file".as_ptr(),
            has_arg: required_argument,
            flag: &raw mut flag,
            val: 7,
        },
        end(),
    ];
    let run = drive(&["tmux", "--file"], c"f:", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 0);
}

/// With one dash accepted for long options, a name that is also a short
/// option stays a short option.
#[test]
fn one_dash_may_be_enough_for_a_long_option() {
    let _guard = parser();
    let table = [long(c"file", no_argument, 'F' as c_int), end()];
    assert_eq!(
        opts(&drive(&["tmux", "-file"], c"fq", &table, FLAG_LONGONLY)),
        [('F', None)]
    );
    assert_eq!(
        opts(&drive(&["tmux", "-f"], c"fq", &table, FLAG_LONGONLY)),
        [('f', None)]
    );
    assert_eq!(
        opts(&drive(&["tmux", "-q"], c"fq", &table, FLAG_LONGONLY)),
        [('q', None)]
    );
}

/// `W;` in the option string asks for `-W name` to be read as the long
/// option `name`.
#[test]
fn a_w_option_stands_for_a_long_option() {
    let _guard = parser();
    let table = [long(c"file", required_argument, 'f' as c_int), end()];
    assert_eq!(
        opts(&drive(&["tmux", "-W", "file=conf"], c"W;", &table, 0)),
        [('f', Some("conf"))]
    );
    assert_eq!(
        opts(&drive(&["tmux", "-Wfile=conf"], c"W;", &table, 0)),
        [('f', Some("conf"))]
    );
    unsafe { BSDopterr = 1 };
    let run = drive(&["tmux", "-W"], c"W;", &table, 0);
    assert_eq!(opts(&run), [('?', None)]);
    assert_eq!(run.optopt, 'W' as c_int);
    assert_eq!(
        opts(&drive(&["tmux", "-W"], c":W;", &table, 0)),
        [(':', None)]
    );
}
