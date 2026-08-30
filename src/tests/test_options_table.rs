use super::{options_other_names, options_table};
use ::core::ffi::CStr;
use ::std::fmt::Write as _;

const OPTIONS_TABLE_LEN: usize = 221;
const OPTIONS_OTHER_NAMES_LEN: usize = 6;

fn text(s: Option<&CStr>) -> String {
    match s {
        None => String::from("<null>"),
        Some(s) => s.to_string_lossy().into_owned(),
    }
}

fn strings(out: &mut String, label: &str, list: Option<&[&CStr]>) {
    let Some(list) = list else {
        let _ = writeln!(out, "  {label}=<null>");
        return;
    };
    for (i, s) in list.iter().enumerate() {
        let _ = writeln!(out, "  {label}[{i}]={}", text(Some(s)));
    }
    let _ = writeln!(out, "  {label}.count={}", list.len());
}

fn dump() -> String {
    let mut out = String::new();
    for (i, e) in options_table.iter().enumerate() {
        let _ = writeln!(out, "entry {i}");
        let _ = writeln!(out, "  name={}", text(Some(e.name)));
        let _ = writeln!(out, "  alternative_name={}", text(e.alternative_name));
        let _ = writeln!(out, "  type={}", e.type_0);
        let _ = writeln!(out, "  scope={}", e.scope);
        let _ = writeln!(out, "  flags={}", e.flags);
        let _ = writeln!(out, "  minimum={}", e.minimum);
        let _ = writeln!(out, "  maximum={}", e.maximum);
        strings(&mut out, "choices", e.choices);
        let _ = writeln!(out, "  default_str={}", text(e.default_str));
        let _ = writeln!(out, "  default_num={}", e.default_num);
        strings(&mut out, "default_arr", e.default_arr);
        let _ = writeln!(out, "  separator={}", text(e.separator));
        let _ = writeln!(out, "  pattern={}", text(e.pattern));
        let _ = writeln!(out, "  text={}", text(e.text));
        let _ = writeln!(out, "  unit={}", text(e.unit));
    }
    for (i, m) in options_other_names.iter().enumerate() {
        let _ = writeln!(out, "other_name {i}");
        let _ = writeln!(out, "  from={}", text(Some(m.from)));
        let _ = writeln!(out, "  to={}", text(Some(m.to)));
    }
    out
}

#[test]
fn table_matches_golden() {
    let got = dump();
    let want = include_str!("../options_table.golden");
    for (n, (a, b)) in got.lines().zip(want.lines()).enumerate() {
        assert_eq!(a, b, "options_table.golden differs at line {}", n + 1);
    }
    assert_eq!(
        got.lines().count(),
        want.lines().count(),
        "options_table.golden has a different number of lines"
    );
}
