#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
repo=$(dirname "$root")
oracle=${TMUX_UPDATE_TARGET:?run inside nix develop}
metadata=$(dirname "$oracle")/../share/tmux
test "$("$oracle" -V)" = 'tmux next-3.9'
test "$(cat "$metadata/hmux-upstream-revision")" = e880cf63e0a9fe095d7c5d313761520fb1a8653c
work=$(mktemp -d /tmp/hmux2-gen.XXXXXX)
trap 'rm -rf "$work"' EXIT
mkdir "$work/source"
tar -xf "$(cat "$metadata/hmux-patched-source")" -C "$work/source"
cd "$repo"
mapfile -t flags < <(nix eval --json .#tmux-target.configureFlags | jq -r '.[]')
cd "$work/source"
sh autogen.sh
./configure "${flags[@]}" CC=clang
bear -- make -j"${JOBS:-4}"
jq '[group_by(.file)[] | max_by(.arguments | length)]' compile_commands.json > "$work/compile_commands.json"
read -ra clang_flags <<< "${NIX_CFLAGS_COMPILE:-}"
c2rust transpile "$work/compile_commands.json" --binary tmux --output-dir "$work/generated" --fail-on-error -- "${clang_flags[@]}"
python3 - "$work" <<'PY'
import json
from pathlib import Path
import re
import sys
work = Path(sys.argv[1])
entries = json.loads((work / "compile_commands.json").read_text())
for entry in entries:
    source = Path(entry["file"])
    translated = work / "generated/src" / source.with_suffix(".rs").as_posix().replace("-", "_")
    if not translated.is_file():
        raise SystemExit(f"missing translation: {translated}")
notices = []
for path in sorted((work / "source").rglob("*")):
    if path.suffix in {".c", ".h", ".y"}:
        for block in re.findall(r"/\*.*?\*/", path.read_text(errors="replace"), re.S):
            if "copyright" in block.lower():
                notices.append(f"{path.relative_to(work / 'source')}\n{block}\n")
(work / "generated/COPYRIGHT").write_text("\n".join(notices))
PY
python3 "$root/scripts/fixups.py" "$work/generated"
cp -R "$work/generated/src" "$root/"
cp "$work/generated/Cargo.toml" "$work/generated/lib.rs" "$root/"
cp "$work/generated/COPYRIGHT" "$root/"
cp "$metadata/hmux-upstream-revision" "$root/UPSTREAM_REVISION"
cp "$work/source/COPYING" "$root/COPYING"
