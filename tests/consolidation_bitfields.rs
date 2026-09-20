//! Compare the generated signed 24-bit stdio field against the captured bytes.
#[test]
fn stdio_bitfield_bytes_and_accessors_match_baseline() {
    let mut expected = None;
    macro_rules! bitfield {
        ($ty:path) => {{
            let mut actual = String::new();
            for value in [0, 1, 0x123456, 0x7fffff, -1, -0x800000, i32::MAX] {
                let mut file: $ty = unsafe { std::mem::zeroed() };
                file._short_backupbuf[0] = 0x5a;
                file.set__flags2(value);
                actual.push_str(&format!(
                    "{value} {} {:?} {}\n",
                    file._flags2(),
                    file._flags2,
                    file._short_backupbuf[0]
                ));
            }
            if let Some(ref old) = expected {
                assert_eq!(old, &actual);
            }
            expected = Some(actual);
        }};
    }
    bitfield!(hmux2::src::cfg::_IO_FILE);
    bitfield!(hmux2::src::client::_IO_FILE);
    bitfield!(hmux2::src::cmd_parse::_IO_FILE);
    bitfield!(hmux2::src::file::_IO_FILE);
    bitfield!(hmux2::src::log::_IO_FILE);
    bitfield!(hmux2::src::osdep_linux::_IO_FILE);
    bitfield!(hmux2::src::prompt_history::_IO_FILE);
    bitfield!(hmux2::src::server::_IO_FILE);
    bitfield!(hmux2::src::spawn::_IO_FILE);
    bitfield!(hmux2::src::tmux::_IO_FILE);
    let actual = expected.unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(
        actual,
        std::fs::read_to_string(root.join("tests/fixtures/consolidation-bitfields.txt")).unwrap()
    );
}
