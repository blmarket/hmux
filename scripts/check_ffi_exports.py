#!/usr/bin/env python3
"""Check every C export retained from the pre-import-migration static library."""
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
archive = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else root / 'target/debug/libhmux2.a'
required = {line.split('\t')[0] for line in
            (root / 'docs/required-exports.tsv').read_text().splitlines()[1:]}
result = subprocess.run(['nm', '-g', '--defined-only', str(archive)],
                        text=True, capture_output=True, check=True)
actual = {fields[2] for line in result.stdout.splitlines()
          if len(fields := line.split()) == 3}
missing = required - actual
if missing:
    sys.exit('Missing required exports:\n' + '\n'.join(sorted(missing)))
print(f'All {len(required)} required C exports are present.')
