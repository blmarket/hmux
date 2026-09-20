//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISalnum;
    records.push(format!(
        "src/arguments.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISalnum;
    records.push(format!(
        "src/cmd_parse.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISalnum;
    records.push(format!(
        "src/cmd_source_file.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISalnum;
    records.push(format!(
        "src/colour.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISalnum;
    records.push(format!(
        "src/compat/vis.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISalnum;
    records.push(format!(
        "src/format.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISalnum;
    records.push(format!(
        "src/json.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISalnum;
    records.push(format!(
        "src/layout_custom.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISalnum;
    records.push(format!(
        "src/names.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISalnum;
    records.push(format!(
        "src/options.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISalnum;
    records.push(format!(
        "src/tty_keys.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISalnum;
    records.push(format!(
        "src/utf8.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISalnum;
    records.push(format!(
        "src/window.rs::_ISalnum {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISalpha;
    records.push(format!(
        "src/arguments.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISalpha;
    records.push(format!(
        "src/cmd_parse.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISalpha;
    records.push(format!(
        "src/cmd_source_file.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISalpha;
    records.push(format!(
        "src/colour.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISalpha;
    records.push(format!(
        "src/compat/vis.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISalpha;
    records.push(format!(
        "src/format.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISalpha;
    records.push(format!(
        "src/json.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISalpha;
    records.push(format!(
        "src/layout_custom.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISalpha;
    records.push(format!(
        "src/names.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISalpha;
    records.push(format!(
        "src/options.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISalpha;
    records.push(format!(
        "src/tty_keys.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISalpha;
    records.push(format!(
        "src/utf8.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISalpha;
    records.push(format!(
        "src/window.rs::_ISalpha {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISblank;
    records.push(format!(
        "src/arguments.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISblank;
    records.push(format!(
        "src/cmd_parse.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISblank;
    records.push(format!(
        "src/cmd_source_file.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISblank;
    records.push(format!(
        "src/colour.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISblank;
    records.push(format!(
        "src/compat/vis.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISblank;
    records.push(format!(
        "src/format.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISblank;
    records.push(format!(
        "src/json.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISblank;
    records.push(format!(
        "src/layout_custom.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISblank;
    records.push(format!(
        "src/names.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISblank;
    records.push(format!(
        "src/options.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISblank;
    records.push(format!(
        "src/tty_keys.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISblank;
    records.push(format!(
        "src/utf8.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISblank;
    records.push(format!(
        "src/window.rs::_ISblank {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_IScntrl;
    records.push(format!(
        "src/arguments.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_IScntrl;
    records.push(format!(
        "src/cmd_parse.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_IScntrl;
    records.push(format!(
        "src/cmd_source_file.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_IScntrl;
    records.push(format!(
        "src/colour.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_IScntrl;
    records.push(format!(
        "src/compat/vis.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_IScntrl;
    records.push(format!(
        "src/format.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_IScntrl;
    records.push(format!(
        "src/json.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_IScntrl;
    records.push(format!(
        "src/layout_custom.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_IScntrl;
    records.push(format!(
        "src/names.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_IScntrl;
    records.push(format!(
        "src/options.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_IScntrl;
    records.push(format!(
        "src/tty_keys.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_IScntrl;
    records.push(format!(
        "src/utf8.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_IScntrl;
    records.push(format!(
        "src/window.rs::_IScntrl {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISdigit;
    records.push(format!(
        "src/arguments.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISdigit;
    records.push(format!(
        "src/cmd_parse.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISdigit;
    records.push(format!(
        "src/cmd_source_file.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISdigit;
    records.push(format!(
        "src/colour.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISdigit;
    records.push(format!(
        "src/compat/vis.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISdigit;
    records.push(format!(
        "src/format.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISdigit;
    records.push(format!(
        "src/json.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISdigit;
    records.push(format!(
        "src/layout_custom.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISdigit;
    records.push(format!(
        "src/names.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISdigit;
    records.push(format!(
        "src/options.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISdigit;
    records.push(format!(
        "src/tty_keys.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISdigit;
    records.push(format!(
        "src/utf8.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISdigit;
    records.push(format!(
        "src/window.rs::_ISdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISgraph;
    records.push(format!(
        "src/arguments.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISgraph;
    records.push(format!(
        "src/cmd_parse.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISgraph;
    records.push(format!(
        "src/cmd_source_file.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISgraph;
    records.push(format!(
        "src/colour.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISgraph;
    records.push(format!(
        "src/compat/vis.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISgraph;
    records.push(format!(
        "src/format.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISgraph;
    records.push(format!(
        "src/json.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISgraph;
    records.push(format!(
        "src/layout_custom.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISgraph;
    records.push(format!(
        "src/names.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISgraph;
    records.push(format!(
        "src/options.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISgraph;
    records.push(format!(
        "src/tty_keys.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISgraph;
    records.push(format!(
        "src/utf8.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISgraph;
    records.push(format!(
        "src/window.rs::_ISgraph {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISlower;
    records.push(format!(
        "src/arguments.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISlower;
    records.push(format!(
        "src/cmd_parse.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISlower;
    records.push(format!(
        "src/cmd_source_file.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISlower;
    records.push(format!(
        "src/colour.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISlower;
    records.push(format!(
        "src/compat/vis.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISlower;
    records.push(format!(
        "src/format.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISlower;
    records.push(format!(
        "src/json.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISlower;
    records.push(format!(
        "src/layout_custom.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISlower;
    records.push(format!(
        "src/names.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISlower;
    records.push(format!(
        "src/options.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISlower;
    records.push(format!(
        "src/tty_keys.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISlower;
    records.push(format!(
        "src/utf8.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISlower;
    records.push(format!(
        "src/window.rs::_ISlower {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISprint;
    records.push(format!(
        "src/arguments.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISprint;
    records.push(format!(
        "src/cmd_parse.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISprint;
    records.push(format!(
        "src/cmd_source_file.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISprint;
    records.push(format!(
        "src/colour.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISprint;
    records.push(format!(
        "src/compat/vis.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISprint;
    records.push(format!(
        "src/format.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISprint;
    records.push(format!(
        "src/json.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISprint;
    records.push(format!(
        "src/layout_custom.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISprint;
    records.push(format!(
        "src/names.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISprint;
    records.push(format!(
        "src/options.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISprint;
    records.push(format!(
        "src/tty_keys.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISprint;
    records.push(format!(
        "src/utf8.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISprint;
    records.push(format!(
        "src/window.rs::_ISprint {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISpunct;
    records.push(format!(
        "src/arguments.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISpunct;
    records.push(format!(
        "src/cmd_parse.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISpunct;
    records.push(format!(
        "src/cmd_source_file.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISpunct;
    records.push(format!(
        "src/colour.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISpunct;
    records.push(format!(
        "src/compat/vis.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISpunct;
    records.push(format!(
        "src/format.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISpunct;
    records.push(format!(
        "src/json.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISpunct;
    records.push(format!(
        "src/layout_custom.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISpunct;
    records.push(format!(
        "src/names.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISpunct;
    records.push(format!(
        "src/options.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISpunct;
    records.push(format!(
        "src/tty_keys.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISpunct;
    records.push(format!(
        "src/utf8.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISpunct;
    records.push(format!(
        "src/window.rs::_ISpunct {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISspace;
    records.push(format!(
        "src/arguments.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISspace;
    records.push(format!(
        "src/cmd_parse.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISspace;
    records.push(format!(
        "src/cmd_source_file.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISspace;
    records.push(format!(
        "src/colour.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISspace;
    records.push(format!(
        "src/compat/vis.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISspace;
    records.push(format!(
        "src/format.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISspace;
    records.push(format!(
        "src/json.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISspace;
    records.push(format!(
        "src/layout_custom.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISspace;
    records.push(format!(
        "src/names.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISspace;
    records.push(format!(
        "src/options.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISspace;
    records.push(format!(
        "src/tty_keys.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISspace;
    records.push(format!(
        "src/utf8.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISspace;
    records.push(format!(
        "src/window.rs::_ISspace {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISupper;
    records.push(format!(
        "src/arguments.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISupper;
    records.push(format!(
        "src/cmd_parse.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISupper;
    records.push(format!(
        "src/cmd_source_file.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISupper;
    records.push(format!(
        "src/colour.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISupper;
    records.push(format!(
        "src/compat/vis.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISupper;
    records.push(format!(
        "src/format.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISupper;
    records.push(format!(
        "src/json.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISupper;
    records.push(format!(
        "src/layout_custom.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISupper;
    records.push(format!(
        "src/names.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISupper;
    records.push(format!(
        "src/options.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISupper;
    records.push(format!(
        "src/tty_keys.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISupper;
    records.push(format!(
        "src/utf8.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISupper;
    records.push(format!(
        "src/window.rs::_ISupper {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let v: hmux2::src::arguments::ctype_code = hmux2::src::arguments::_ISxdigit;
    records.push(format!(
        "src/arguments.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    let v: hmux2::src::cmd_parse::ctype_code = hmux2::src::cmd_parse::_ISxdigit;
    records.push(format!(
        "src/cmd_parse.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    let v: hmux2::src::cmd_source_file::ctype_code = hmux2::src::cmd_source_file::_ISxdigit;
    records.push(format!(
        "src/cmd_source_file.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    let v: hmux2::src::colour::ctype_code = hmux2::src::colour::_ISxdigit;
    records.push(format!(
        "src/colour.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    let v: hmux2::src::compat::vis::ctype_code = hmux2::src::compat::vis::_ISxdigit;
    records.push(format!(
        "src/compat/vis.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    let v: hmux2::src::format::ctype_code = hmux2::src::format::_ISxdigit;
    records.push(format!(
        "src/format.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    let v: hmux2::src::json::ctype_code = hmux2::src::json::_ISxdigit;
    records.push(format!(
        "src/json.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    let v: hmux2::src::layout_custom::ctype_code = hmux2::src::layout_custom::_ISxdigit;
    records.push(format!(
        "src/layout_custom.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    let v: hmux2::src::names::ctype_code = hmux2::src::names::_ISxdigit;
    records.push(format!(
        "src/names.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    let v: hmux2::src::options::ctype_code = hmux2::src::options::_ISxdigit;
    records.push(format!(
        "src/options.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    let v: hmux2::src::tty_keys::ctype_code = hmux2::src::tty_keys::_ISxdigit;
    records.push(format!(
        "src/tty_keys.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    let v: hmux2::src::utf8::ctype_code = hmux2::src::utf8::_ISxdigit;
    records.push(format!(
        "src/utf8.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    let v: hmux2::src::window::ctype_code = hmux2::src::window::_ISxdigit;
    records.push(format!(
        "src/window.rs::_ISxdigit {:?} {} {}",
        v,
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    records.push(format!(
        "src/arguments.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::arguments::ctype_code>(),
        align_of::<hmux2::src::arguments::ctype_code>()
    ));
    records.push(format!(
        "src/cmd_parse.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::cmd_parse::ctype_code>(),
        align_of::<hmux2::src::cmd_parse::ctype_code>()
    ));
    records.push(format!(
        "src/cmd_source_file.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::cmd_source_file::ctype_code>(),
        align_of::<hmux2::src::cmd_source_file::ctype_code>()
    ));
    records.push(format!(
        "src/colour.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::colour::ctype_code>(),
        align_of::<hmux2::src::colour::ctype_code>()
    ));
    records.push(format!(
        "src/compat/vis.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::compat::vis::ctype_code>(),
        align_of::<hmux2::src::compat::vis::ctype_code>()
    ));
    records.push(format!(
        "src/format.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::format::ctype_code>(),
        align_of::<hmux2::src::format::ctype_code>()
    ));
    records.push(format!(
        "src/json.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::json::ctype_code>(),
        align_of::<hmux2::src::json::ctype_code>()
    ));
    records.push(format!(
        "src/layout_custom.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::layout_custom::ctype_code>(),
        align_of::<hmux2::src::layout_custom::ctype_code>()
    ));
    records.push(format!(
        "src/names.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::names::ctype_code>(),
        align_of::<hmux2::src::names::ctype_code>()
    ));
    records.push(format!(
        "src/options.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::options::ctype_code>(),
        align_of::<hmux2::src::options::ctype_code>()
    ));
    records.push(format!(
        "src/tty_keys.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::tty_keys::ctype_code>(),
        align_of::<hmux2::src::tty_keys::ctype_code>()
    ));
    records.push(format!(
        "src/utf8.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::utf8::ctype_code>(),
        align_of::<hmux2::src::utf8::ctype_code>()
    ));
    records.push(format!(
        "src/window.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::window::ctype_code>(),
        align_of::<hmux2::src::window::ctype_code>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-ctype.txt"));
}
