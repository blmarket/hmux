from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
for path in root.rglob("*.rs"):
    text = path.read_text()
    text = text.replace("::core::ffi::VaListImpl", "::core::ffi::VaList")
    text = text.replace(".as_va_list()", "")
    text = text.replace("use ::generated;", "use ::hmux2;")
    text = text.replace("::std::env::args()", "::std::env::args_os()")
    text = text.replace(
        "::std::ffi::CString::new(arg)",
        "::std::ffi::CString::new(::std::os::unix::ffi::OsStrExt::as_bytes(arg.as_os_str()))",
    )
    path.write_text(text)

manifest = root / "Cargo.toml"
text = re.sub(r'^name = "[^"]+"', 'name = "hmux2"', manifest.read_text(), flags=re.M)
manifest.write_text(text)
