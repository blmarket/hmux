#!/usr/bin/env python3
"""Compiler-assisted inventory of model field accesses across owner boundaries.

Probe a disposable source copy with private model fields. Rust resolves aliases,
raw dereferences and references passed to helpers; source spelling of .get() is
irrelevant. The real worktree is never changed. Nonzero means violations remain.
This complements, rather than replaces, the syntax checks in model_trait_boundary:
field privacy alone cannot detect a reference/pointer forwarded without field use.
"""
import argparse
from functools import lru_cache
from collections import Counter
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MODELS = {
    'session': 'src/session/model.rs',
    'window': 'src/shared/window.rs',
    'window_pane': 'src/shared/pane.rs',
    'client': 'src/shared/client.rs',
}


@lru_cache(maxsize=None)
def mask(source):
    """Keep offsets/lines, masking comments and literals (including raw strings)."""
    result = list(source)
    raw_literal = re.compile(r'(?:br|r)(\#*)"')
    character = re.compile(r"'(?:\\u\{[\da-fA-F_]+\}|\\x[\da-fA-F]{2}|\\.|[^'\\\n])'")
    i = 0
    while i < len(source):
        end = None
        if source.startswith('//', i):
            end = source.find('\n', i)
            if end < 0:
                end = len(source)
        elif source.startswith('/*', i):
            depth, j = 1, i + 2
            while depth and j < len(source):
                if source.startswith('/*', j):
                    depth += 1
                    j += 2
                elif source.startswith('*/', j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            end = j
        else:
            raw = raw_literal.match(source, i)
            if raw:
                closing = '"' + raw[1]
                j = source.find(closing, raw.end())
                end = len(source) if j < 0 else j + len(closing)
            elif source[i] == '"':
                j = i + 1
                while j < len(source):
                    if source[j] == '\\':
                        j += 2
                    elif source[j] == '"':
                        j += 1
                        break
                    else:
                        j += 1
                end = j
            elif source[i] == "'":
                char = character.match(source, i)
                if char:
                    end = char.end()
        if end is None:
            i += 1
        else:
            for j in range(i, end):
                if result[j] != '\n':
                    result[j] = ' '
            i = end
    return ''.join(result)


@lru_cache(maxsize=None)
def function_ranges(source):
    code = mask(source)
    ranges = []
    for function in re.finditer(r'\bfn\s+(\w+)\s*[<(]', code):
        start = code.find('{', function.end())
        semi = code.find(';', function.end())
        if start < 0 or (semi >= 0 and semi < start):
            continue
        depth, end = 1, start + 1
        while depth and end < len(code):
            depth += (code[end] == '{') - (code[end] == '}')
            end += 1
        ranges.append((function.start(), end, function[1]))
    return ranges


def enclosing_function(source, offset):
    found = '<item>'
    for start, end, name in function_ranges(source):
        if start <= offset < end:
            found = name
    return found


def owner(path, function):
    # One entity per implementation. In particular, window.rs is NOT exempt:
    # Pane helpers must still use Window for their parent, and vice versa.
    if path == 'src/session.rs' or path.startswith('src/session/'):
        return 'session'
    if path in ('src/server_client.rs', 'src/server_client/api.rs'):
        return 'client'
    if path in ('src/window/api.rs', 'src/window/alerts.rs'):
        return 'window'
    if path in ('src/window/pane_api.rs', 'src/window/pane_sync.rs'):
        return 'window_pane'
    if path == 'src/window.rs':
        if function in ('window_pane_first', 'window_pane_last', 'window_pane_count'):
            return 'window'
        if function.startswith('window_pane_'):
            return 'window_pane'
        if function.startswith('window_'):
            return 'window'
    return None


def private_fields(source, model):
    code = mask(source)
    match = re.search(r'pub struct ' + model + r'\s*\{', code)
    assert match, model
    end = code.index('\n}', match.end())
    fields = source[match.end():end]
    fields = re.sub(r'(?m)^(\s*)pub(?:\([^)]*\))? ', r'\1', fields)
    return source[:match.end()] + fields + source[end:]


def classify(messages, sources):
    violations, internal, unexpected = [], Counter(), []
    pattern = re.compile(r'field `([^`]+)` of struct `([^`]+)` is private')
    for message in messages:
        if message.get('level') != 'error':
            continue
        match = pattern.fullmatch(message['message'])
        model = match[2].split('::')[-1] if match else None
        if model not in MODELS:
            # Type inference can produce follow-up errors after privacy errors.
            unexpected.append(message['message'])
            continue
        span = next(s for s in message['spans'] if s['is_primary'])
        path = span['file_name']
        source = sources.get(path, '')
        # rustc uses byte offsets, while the lexical scanner uses Unicode offsets.
        offset = len(source.encode()[:span['byte_start']].decode())
        function = enclosing_function(source, offset)
        if owner(path, function) == model:
            internal[model] += 1
        else:
            violations.append(dict(file=path, line=span['line_start'], function=function,
                                   model=model, field=match[1]))
    return violations, internal, unexpected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--json', type=Path, help='write all findings for review')
    parser.add_argument('--model', action='append', choices=MODELS,
                        help='probe only this model (repeatable; default: all four)')
    args = parser.parse_args()
    selected = args.model or list(MODELS)
    sources = {str(p.relative_to(ROOT)): p.read_text() for p in (ROOT / 'src').rglob('*.rs')}
    with tempfile.TemporaryDirectory(prefix='hmux-model-boundary-') as directory:
        probe = Path(directory)
        for name in ('src', '.cargo', 'hmux-buffer', 'hmux-cmdparse', 'hmux-refbox', 'hmux-rt'):
            shutil.copytree(ROOT / name, probe / name, ignore=shutil.ignore_patterns('target', '.git'))
        for name in ('Cargo.toml', 'Cargo.lock', 'build.rs'):
            shutil.copy2(ROOT / name, probe / name)
        for model in selected:
            path = MODELS[model]
            (probe / path).write_text(private_fields(sources[path], model))
        # Preserve the probed offsets: removing pub changes columns/byte offsets.
        probe_sources = {p: (probe / p).read_text() for p in sources}
        result = subprocess.run(['cargo', 'check', '--offline', '--lib', '--message-format=json',
                                 '--target-dir', str(ROOT / 'target/model-boundary')],
                                cwd=probe, capture_output=True, text=True)
        messages = [item['message'] for line in result.stdout.splitlines()
                    if (item := json.loads(line)).get('reason') == 'compiler-message']
        violations, internal, unexpected = classify(messages, probe_sources)
    failed_without_errors = result.returncode and not any(
        message.get('level') == 'error' for message in messages)
    if failed_without_errors:
        unexpected.append(f'cargo failed with exit {result.returncode} without error diagnostics')
    report = dict(models=selected, violations=violations, internal=dict(internal),
                  other_errors=unexpected)
    if args.json:
        args.json.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Candidate external model field accesses: {len(violations)}')
    for model, count in sorted(Counter(v['model'] for v in violations).items()):
        print(f'  {model}: {count}')
    print(f'Owner implementation accesses: {dict(internal)}')
    for path, count in Counter(v['file'] for v in violations).most_common(15):
        print(f'  {path}: {count}')
    if unexpected:
        print(f'Other compiler errors (including follow-up inference errors): {len(unexpected)}')
    if not messages and result.returncode:
        print(result.stderr)
    return 1 if violations or unexpected or (not messages and result.returncode) else 0


if __name__ == '__main__':
    raise SystemExit(main())
