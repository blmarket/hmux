#!/usr/bin/env python3
"""Build and compare reproducible checkpoints for the format.rs split.

The staged checkpoints are materialized from the immutable pre-split source.
Each checkpoint adds exactly one private implementation module and its facade
imports.  This preserves a one-group-at-a-time build/test/Clippy comparison
even though the historical split commit was monolithic and must not be
rewritten.  The checked-in final revision is copied as the final checkpoint.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATION = ROOT / "target" / "format-validation"
CHECKPOINTS = VALIDATION / "format-checkpoints"
ARCHIVES = VALIDATION / "format-checkpoint-archives"
LOGS = VALIDATION / "format-checkpoint-logs"
TARGETS = VALIDATION / "format-checkpoint-targets"
# These are deliberately immutable source points.  The baseline is the last
# revision before this split, and SPLIT_REV is the commit containing the four
# moved implementation files.  FINAL_REV is the checked-in source being
# validated; prepare() records its resolved SHA in the manifest.
BASELINE_REV = "8d02dca179520da8b25ba6cb1de3e6a54dafd490"
SPLIT_REV = "cc5b93feb4395cda2edc8de3463bd04a312ec94d"
FINAL_REV = "fe16550d49a14bfdb2485cb663387c403d2f5049"
STAGES = ("baseline", "tree", "expression", "jobs", "callbacks", "final")
GROUPS = ("tree", "expression", "jobs", "callbacks")

COMMANDS = {
    "build": ["cargo", "build"],
    "test": ["cargo", "test"],
    "clippy": [
        "cargo",
        "clippy",
        "--all-targets",
        "--message-format=json",
        "--",
        "-D",
        "warnings",
    ],
}

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


def split_source(group: str) -> bytes:
    return subprocess.check_output(
        ["git", "show", f"{SPLIT_REV}:src/format/{group}.rs"], cwd=ROOT
    )


def format_checkpoint(
    destination: Path,
    moved: tuple[str, ...],
    items: dict[str, list[str]],
    implementation_sources: dict[str, bytes],
) -> None:
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
        target.write_bytes(implementation_sources[group])


def digest_files(path: Path, suffix: str | None = None) -> str:
    digest = hashlib.sha256()
    files = sorted(path.rglob("*"))
    for file in files:
        if not file.is_file() or suffix is not None and file.suffix != suffix:
            continue
        digest.update(file.relative_to(path).as_posix().encode())
        digest.update(file.read_bytes())
    return digest.hexdigest()


def write_checkpoint_archive(stage: str) -> str:
    """Write a deterministic tar snapshot and return its digest."""

    ARCHIVES.mkdir(parents=True, exist_ok=True)
    archive_path = ARCHIVES / f"{stage}.tar"
    with tarfile.open(archive_path, mode="w") as archive:
        root = CHECKPOINTS / stage
        for file in sorted(root.rglob("*")):
            if not file.is_file():
                continue
            relative = file.relative_to(root).as_posix()
            info = archive.gettarinfo(str(file), arcname=relative)
            info.uid = 0
            info.gid = 0
            info.uname = ""
            info.gname = ""
            info.mtime = 0
            with file.open("rb") as stream:
                archive.addfile(info, stream)
    return hashlib.sha256(archive_path.read_bytes()).hexdigest()


def prepare() -> dict:
    items = group_items()
    implementation_sources = {group: split_source(group) for group in GROUPS}
    for group, expected in implementation_sources.items():
        actual = (ROOT / "src" / "format" / f"{group}.rs").read_bytes()
        if actual != expected:
            raise RuntimeError(
                f"{group}.rs differs from immutable split revision {SPLIT_REV}"
            )
    if CHECKPOINTS.exists():
        shutil.rmtree(CHECKPOINTS)
    if ARCHIVES.exists():
        shutil.rmtree(ARCHIVES)
    CHECKPOINTS.mkdir(parents=True)

    git_archive(BASELINE_REV, CHECKPOINTS / "baseline")
    for index, group in enumerate(GROUPS, start=1):
        stage = GROUPS[:index]
        destination = CHECKPOINTS / group
        git_archive(BASELINE_REV, destination)
        format_checkpoint(destination, stage, items, implementation_sources)
    git_archive(FINAL_REV, CHECKPOINTS / "final")

    baseline_sha = subprocess.check_output(
        ["git", "rev-parse", BASELINE_REV], cwd=ROOT, text=True
    ).strip()
    split_sha = subprocess.check_output(
        ["git", "rev-parse", SPLIT_REV], cwd=ROOT, text=True
    ).strip()
    final_sha = subprocess.check_output(
        ["git", "rev-parse", FINAL_REV], cwd=ROOT, text=True
    ).strip()
    stage_metadata = {}
    for stage in STAGES:
        stage_path = CHECKPOINTS / stage
        stage_metadata[stage] = {
            "files_sha256": digest_files(stage_path),
            "rust_files_sha256": digest_files(stage_path, ".rs"),
            "archive_sha256": write_checkpoint_archive(stage),
        }
    manifest = {
        "schema": 2,
        "baseline_revision": baseline_sha,
        "split_revision": split_sha,
        "final_revision": final_sha,
        "stage_order": STAGES,
        "group_items": items,
        "stages": stage_metadata,
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
    environment = os.environ.copy()
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
    manifest = prepare()
    if LOGS.exists():
        shutil.rmtree(LOGS)
    if TARGETS.exists():
        shutil.rmtree(TARGETS)
    results: dict[str, dict] = {}
    for stage in STAGES:
        results[stage] = {
            "commands": {name: shlex.join(command) for name, command in COMMANDS.items()},
            "build": run_command(stage, "build", COMMANDS["build"]),
            "test": run_command(stage, "test", COMMANDS["test"]),
            "clippy": run_command(stage, "clippy-strict-json", COMMANDS["clippy"]),
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
    for index, stage in enumerate(STAGES[1:], start=1):
        before = STAGES[index - 1]
        results[stage]["comparison_to_previous"] = {
            "before_stage": before,
            "after_stage": stage,
            "build": {
                "before_exit": results[before]["build"],
                "after_exit": results[stage]["build"],
                "unchanged": results[before]["build"] == results[stage]["build"],
            },
            "test": {
                "before_exit": results[before]["test"],
                "after_exit": results[stage]["test"],
                "before_passed": results[before]["test_counts"]["passed"],
                "after_passed": results[stage]["test_counts"]["passed"],
                "passed_delta": (
                    results[stage]["test_counts"]["passed"]
                    - results[before]["test_counts"]["passed"]
                ),
            },
            "clippy": {
                "before_exit": results[before]["clippy"],
                "after_exit": results[stage]["clippy"],
                "diagnostics_same": (
                    results[before]["clippy_diagnostics"]["sha256"]
                    == results[stage]["clippy_diagnostics"]["sha256"]
                ),
                "before_count": results[before]["clippy_diagnostics"]["count"],
                "after_count": results[stage]["clippy_diagnostics"]["count"],
            },
        }
    summary = VALIDATION / "format-checkpoints-summary.json"
    summary.write_text(
        json.dumps(
            {
                "baseline_revision": manifest["baseline_revision"],
                "split_revision": manifest["split_revision"],
                "final_revision": manifest["final_revision"],
                "stage_order": STAGES,
                "commands": {name: shlex.join(command) for name, command in COMMANDS.items()},
                "stages": results,
            },
            indent=2,
        )
        + "\n"
    )
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
