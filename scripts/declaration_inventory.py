#!/usr/bin/env python3
"""Inventory translated Rust declarations, without equating anonymous C names.

This intentionally handles the generated declaration syntax in this repository,
not arbitrary Rust. Offsets are used by the one-family-at-a-time migration audit.
"""
import collections
import hashlib
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]


def declarations(text):
    for match in re.finditer(r'(?m)^[ \t]*pub (struct|union|type|const) (\w+)\b', text):
        kind, name = match.groups()
        start = match.start()
        if kind in ('struct', 'union'):
            end = text.index('\n}', match.end()) + 2
            # Preserve representation, derives, and bitfield attributes verbatim.
            while start > 0:
                previous = text.rfind('\n', 0, start - 1) + 1
                if not text[previous:start].startswith('#['):
                    break
                start = previous
        else:
            depth = 0
            quoted = False
            escaped = False
            for end in range(match.end(), len(text)):
                c = text[end]
                if quoted:
                    if escaped:
                        escaped = False
                    elif c == '\\':
                        escaped = True
                    elif c == '"':
                        quoted = False
                elif c == '"':
                    quoted = True
                elif c in '([{':
                    depth += 1
                elif c in ')]}':
                    depth -= 1
                elif c == ';' and depth == 0:
                    end += 1
                    break
        yield dict(kind=kind, name=name, start=start, end=end, text=text[start:end])


def inventory():
    result = collections.defaultdict(list)
    for path in sorted((ROOT / 'src').rglob('*.rs')):
        text = path.read_text()
        for d in declarations(text):
            signature = re.sub(r'\s+', '', d['text'])
            result[d['kind'], d['name']].append({
                'file': str(path.relative_to(ROOT)),
                'line': text.count('\n', 0, d['start']) + 1,
                'sha256': hashlib.sha256(signature.encode()).hexdigest(),
            })
    return [dict(kind=k, name=n, count=len(v), variants=len({x['sha256'] for x in v}), locations=v)
            for (k, n), v in sorted(result.items())]


if __name__ == '__main__':
    print(json.dumps(inventory(), indent=2))
