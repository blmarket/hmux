#!/usr/bin/env python3
"""Build and compare reproducible checkpoints for the format.rs split.

The staged checkpoints are made from the pre-split commit.  Each checkpoint
adds exactly one private implementation module and its facade imports.  The
checked-in HEAD is also copied as a final checkpoint so the report covers the
source that is actually delivered.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tarfile
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATION = ROOT / "target" / "format-validation"
CHECKPOINTS = VALIDATION / "format-checkpoints"
LOGS = VALIDATION / "format-checkpoint-logs"
TARGETS = VALIDATION / "format-checkpoint-targets"
BASELINE_REV = "cc5b93f^"
FINAL_REV = "HEAD"
STAGES = ("baseline", "tree", "expression", "jobs", "callbacks", "final")
GROUPS = ("tree", "expression", "jobs", "callbacks")

PUBLIC_EXPORTS = {
    "tree": (
        "format_add",
        "format_add_cb",
        "format_add_tv",
        "format_create",
        "format_each",
        "format_free",
        "format_get_pane",
        "format_log_debug",
        "format_merge",
    ),
    "expression": (
        "format_expand",
        "format_expand_time",
        "format_pretty_time",
        "format_single",
        "format_single_from_state",
        "format_single_from_target",
        "format_skip",
        "format_true",
    ),
    "jobs": ("format_lost_client", "format_tidy_jobs"),
    "callbacks": (),
}

GROUP_COMMENTS = {
    "tree": """/* Private tree-storage checkpoint. */\n""",
    "expression": """/* Private expression checkpoint. */\n""",
    "jobs": """/* Private job-integration checkpoint. */\n""",
    "callbacks": """/* Private default-callback checkpoint. */\n""",
}


def git_archive(revision: str, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=False)
    archive = subprocess.Popen(
        ["git", "archive", revision], cwd=ROOT, stdout=subprocess.PIPE
    )
    assert archive.stdout is not None
    with tarfile.open(fileobj=archive.stdout, mode="r|*") as stream:
        stream.extractall(destination)
    status = archive.wait()
    if status != 0:
        raise RuntimeError(f"git archive {revision} failed with status {status}")


def top_level_items(source: str) -> list[str]:
    """Return function/static names declared at column zero in a module."""

    names: list[str] = []
    pattern = re.compile(
        r'^(?:(?:pub(?:\([^)]*\))?|unsafe|extern "C")\s+)*'
        r'(?:fn|static(?:\s+mut)?)\s+([A-Za-z_][A-Za-z0-9_]*)\b',
        re.MULTILINE,
    )
    for match in pattern.finditer(source):
        names.append(match.group(1))
    return names


def code_brace(source: str, start: int) -> int:
    """Find the first non-comment/string brace after start."""

    i = start
    state = "normal"
    raw_hashes = 0
    while i < len(source):
        char = source[i]
        next_char = source[i + 1] if i + 1 < len(source) else ""
        if state == "normal":
            if char == "/" and next_char == "/":
                state = "line-comment"
                i += 2
                continue
            if char == "/" and next_char == "*":
                state = "block-comment"
                i += 2
                continue
            if char == '"':
                state = "string"
                i += 1
                continue
            if char == "'":
                state = "char"
                i += 1
                continue
            if char == "{":
                return i
            i += 1
            continue
        if state == "line-comment":
            if char == "\n":
                state = "normal"
            i += 1
            continue
        if state == "block-comment":
            if char == "*" and next_char == "/":
                state = "normal"
                i += 2
            else:
                i += 1
            continue
        if state == "string":
            if char == "\\":
                i += 2
            elif char == '"':
                state = "normal"
                i += 1
            else:
                i += 1
            continue
        if state == "char":
            if char == "\\":
                i += 2
            elif char == "'":
                state = "normal"
                i += 1
            else:
                i += 1
            continue
    raise ValueError("could not find a Rust item brace")


def item_span(source: str, name: str) -> tuple[int, int]:
    declaration = re.compile(
        r'(?m)^(?:(?:pub(?:\([^)]*\))?|unsafe|extern "C")\s+)*'
        rf'(?:fn|static(?:\s+mut)?)\s+{re.escape(name)}\b'
    )
    match = declaration.search(source)
    if match is None:
        raise ValueError(f"{name} is missing from the pre-split format.rs")
    start = source.rfind("\n", 0, match.start()) + 1
    while start > 0:
        previous_end = start - 1
        previous_start = source.rfind("\n", 0, previous_end) + 1
        previous = source[previous_start:previous_end].strip()
        if previous.startswith("#[") and previous.endswith("]"):
            start = previous_start
        else:
            break

    opening = code_brace(source, match.end())
    depth = 0
    i = opening
    state = "normal"
    while i < len(source):
        char = source[i]
        next_char = source[i + 1] if i + 1 < len(source) else ""
        if state == "normal":
            if char == "/" and next_char == "/":
                state = "line-comment"
                i += 2
                continue
            if char == "/" and next_char == "*":
                state = "block-comment"
                i += 2
                continue
            if char == '"':
                state = "string"
                i += 1
                continue
            if char == "'":
                state = "char"
                i += 1
                continue
            if char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
                if depth == 0:
                    end = i + 1
                    while end < len(source) and source[end] in " \t":
                        end += 1
                    if end < len(source) and source[end] == ";":
                        end += 1
                    if end < len(source) and source[end] == "\n":
                        end += 1
                    return start, end
            i += 1
            continue
        if state == "line-comment":
            if char == "\n":
                state = "normal"
            i += 1
            continue
        if state == "block-comment":
            if char == "*" and next_char == "/":
                state = "normal"
                i += 2
            else:
                i += 1
            continue
        if state == "string":
            if char == "\\":
                i += 2
            elif char == '"':
                state = "normal"
                i += 1
            else:
                i += 1
            continue
        if state == "char":
            if char == "\\":
                i += 2
            elif char == "'":
                state = "normal"
                i += 1
            else:
                i += 1
            continue
    raise ValueError(f"could not find the end of {name}")


def group_items() -> dict[str, list[str]]:
    result: dict[str, list[str]] = {}
    for group in GROUPS:
        path = ROOT / "src" / "format" / f"{group}.rs"
        names = top_level_items(path.read_text())
        if not names:
            raise RuntimeError(f"no top-level items found in {path}")
        result[group] = names
    return result


def facade_stanzas(moved: tuple[str, ...]) -> str:
    pieces: list[str] = []
    for group in GROUPS:
        if group not in moved:
            continue
        pieces.append(GROUP_COMMENTS[group])
        pieces.append(f"mod {group};\nuse {group}::*;\n")
        exports = PUBLIC_EXPORTS[group]
        if exports:
            pieces.append(f"pub use {group}::{{\n    {', '.join(exports)},\n}};\n")
    return "\n".join(pieces)


def format_checkpoint(destination: Path, moved: tuple[str, ...], items: dict[str, list[str]]) -> None:
    source_path = destination / "src" / "format.rs"
    source = source_path.read_text()
    spans: list[tuple[int, int]] = []
    for group in moved:
        for name in items[group]:
            spans.append(item_span(source, name))
    for start, end in sorted(spans, reverse=True):
        source = source[:start] + source[end:]

    marker = "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct format_modifier"
    if marker not in source:
        raise RuntimeError("format facade insertion marker is missing")
    source = source.replace(marker, facade_stanzas(moved) + "\n" + marker, 1)
    source_path.write_text(source)

    for group in moved:
        target = destination / "src" / "format" / f"{group}.rs"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / "src" / "format" / f"{group}.rs", target)


def source_digest(path: Path) -> str:
    digest = hashlib.sha256()
    for file in sorted(path.rglob("*.rs")):
        digest.update(file.relative_to(path).as_posix().encode())
        digest.update(file.read_bytes())
    return digest.hexdigest()


def prepare() -> dict:
    items = group_items()
    if CHECKPOINTS.exists():
        shutil.rmtree(CHECKPOINTS)
    CHECKPOINTS.mkdir(parents=True)

    git_archive(BASELINE_REV, CHECKPOINTS / "baseline")
    for index, group in enumerate(GROUPS, start=1):
        stage = GROUPS[:index]
        destination = CHECKPOINTS / group
        git_archive(BASELINE_REV, destination)
        format_checkpoint(destination, stage, items)
    git_archive(FINAL_REV, CHECKPOINTS / "final")

    manifest = {
        "baseline_revision": subprocess.check_output(
            ["git", "rev-parse", BASELINE_REV], cwd=ROOT, text=True
        ).strip(),
        "final_revision": subprocess.check_output(
            ["git", "rev-parse", FINAL_REV], cwd=ROOT, text=True
        ).strip(),
        "group_items": items,
        "stages": {
            stage: source_digest(CHECKPOINTS / stage) for stage in STAGES
        },
    }
    VALIDATION.mkdir(parents=True, exist_ok=True)
    (VALIDATION / "format-checkpoints.json").write_text(
        json.dumps(manifest, indent=2) + "\n"
    )
    return manifest


def run_command(stage: str, label: str, command: list[str]) -> int:
    LOGS.mkdir(parents=True, exist_ok=True)
    stage_logs = LOGS / stage
    stage_logs.mkdir(parents=True, exist_ok=True)
    target = TARGETS / stage
    target.mkdir(parents=True, exist_ok=True)
    log_path = stage_logs / f"{label}.log"
    environment = dict(**__import__("os").environ)
    environment["CARGO_TARGET_DIR"] = str(target)
    with log_path.open("w") as log:
        result = subprocess.run(
            command,
            cwd=CHECKPOINTS / stage,
            env=environment,
            stdout=log,
            stderr=subprocess.STDOUT,
            text=True,
        )
    return result.returncode


def result_counts(log_path: Path) -> dict:
    text = log_path.read_text(errors="replace")
    passed = [int(value) for value in re.findall(r"test result: ok\. (\d+) passed", text)]
    warning_matches = [
        int(value)
        for value in re.findall(r"generated (\d+) warnings?", text)
    ]
    clippy_totals = [
        int(value)
        for value in re.findall(r"due to (\d+) previous errors", text)
    ]
    return {
        "passed": sum(passed),
        "test_suites": len(passed),
        "generated_warnings": sum(warning_matches),
        "clippy_error_totals": clippy_totals,
    }


def normalized_diagnostics(path: Path) -> list[dict]:
    records: list[dict] = []
    for line in path.read_text(errors="replace").splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("reason") != "compiler-message":
            continue
        message = record.get("message", {})
        code = message.get("code") or {}
        records.append(
            {
                "level": message.get("level"),
                "code": code.get("code"),
                "message": message.get("message"),
            }
        )
    return sorted(records, key=lambda item: json.dumps(item, sort_keys=True))


def diagnostic_digest(records: list[dict]) -> str:
    encoded = json.dumps(records, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def validate() -> dict[str, dict]:
    prepare()
    if LOGS.exists():
        shutil.rmtree(LOGS)
    if TARGETS.exists():
        shutil.rmtree(TARGETS)
    results: dict[str, dict] = {}
    for stage in STAGES:
        results[stage] = {
            "build": run_command(stage, "build", ["cargo", "build"]),
            "test": run_command(stage, "test", ["cargo", "test"]),
            "clippy": run_command(
                stage,
                "clippy-strict-json",
                [
                    "cargo",
                    "clippy",
                    "--all-targets",
                    "--message-format=json",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
        }
        results[stage].update(
            {
                "build_counts": result_counts(LOGS / stage / "build.log"),
                "test_counts": result_counts(LOGS / stage / "test.log"),
                "clippy_counts": result_counts(
                    LOGS / stage / "clippy-strict-json.log"
                ),
            }
        )
        diagnostics = normalized_diagnostics(LOGS / stage / "clippy-strict-json.log")
        results[stage]["clippy_diagnostics"] = {
            "count": len(diagnostics),
            "by_level": dict(Counter(item["level"] for item in diagnostics)),
            "sha256": diagnostic_digest(diagnostics),
        }

    baseline_digest = results["baseline"]["clippy_diagnostics"]["sha256"]
    for stage in STAGES:
        results[stage]["clippy_diagnostics"]["matches_baseline"] = (
            results[stage]["clippy_diagnostics"]["sha256"] == baseline_digest
        )
    summary = VALIDATION / "format-checkpoints-summary.json"
    summary.write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))
    return results


def inventory() -> None:
    items = group_items()
    for group in GROUPS:
        print(f"{group}\t{len(items[group])}\t{','.join(items[group])}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("prepare", "validate", "inventory"))
    args = parser.parse_args()
    if args.command == "prepare":
        print(json.dumps(prepare(), indent=2))
    elif args.command == "validate":
        validate()
    else:
        inventory()
    return 0


if __name__ == "__main__":
    sys.exit(main())
