#!/usr/bin/env python3
"""Exercise an isolated server; emit deterministic JSON for before/after diffing."""
import json
import os
import pathlib
import subprocess
import shlex
import tempfile

root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get('HMUX_BINARY', str(root / 'target/debug/hmux2'))).resolve()
results = []
work = root / 'target/consolidation'
work.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='cli-', dir=work) as tmp:
    env = dict(os.environ, TERM='xterm-256color', LC_ALL='C', TMUX='', SHELL='/bin/sh')
    socket = str(pathlib.Path(tmp) / 'socket')
    base = [str(binary), '-S', socket, '-f', '/dev/null']

    def run(*args, ok=True):
        p = subprocess.run(base + list(args), env=env, capture_output=True, text=True, timeout=20)
        results.append(dict(args=args, status=p.returncode, stdout=p.stdout, stderr=p.stderr))
        if ok and p.returncode:
            raise RuntimeError(results[-1])
        return p.stdout

    try:
        run('-V')
        run('new-session', '-d', '-s', 'regression', '-x', '80', '-y', '24', 'sleep 60')
        run('set-option', '-g', 'remain-on-exit', 'on')
        assert run('display-message', '-p', '#{session_name}:#{window_width}x#{window_height}') == 'regression:80x24\n'
        run('split-window', '-h', '-d', 'sleep 60')
        run('select-layout', 'even-horizontal')
        run('list-panes', '-F', '#{pane_index}:#{pane_width}x#{pane_height}:#{pane_active}')
        run('resize-pane', '-t', '0', '-x', '30')
        run('list-panes', '-F', '#{pane_index}:#{pane_width}x#{pane_height}:#{pane_active}')
        run('set-buffer', '-b', 'probe', 'alpha\nbeta')
        assert run('show-buffer', '-b', 'probe') == 'alpha\nbeta'
        run('set-option', '-g', 'status-style', 'fg=red,bg=blue,bold')
        run('show-options', '-gv', 'status-style')
        run('bind-key', '-T', 'regression', 'C-a', 'display-message', 'bound')
        run('list-keys', '-T', 'regression')
        run('display-message', '-p', '#{pane_width}:#{pane_height}:#{window_panes}')
        run('capture-pane', '-p', '-t', '0')
        producer = (
            "printf '\\033[31mRED\\033[0m\\nplain\\n\\033]8;;https://example.invalid\\007link\\033]8;;\\007\\n\\033]2;probe-title\\007'; "
            + shlex.quote(str(binary)) + ' -S ' + shlex.quote(socket)
            + ' wait-for -S output-ready; sleep 60'
        )
        # The producer includes the temporary socket; keep it out of the transcript.
        subprocess.run(base + ['new-window', '-d', '-n', 'output', producer],
                       env=env, capture_output=True, text=True, check=True, timeout=20)
        run('wait-for', 'output-ready')
        output = run('capture-pane', '-p', '-t', 'regression:output.0')
        assert output.startswith('RED\nplain\nlink\n'), repr(output)
        run('capture-pane', '-p', '-e', '-t', 'regression:output.0')
        assert run('display-message', '-p', '-t', 'regression:output.0', '#{pane_title}') == 'probe-title\n'

        run('definitely-invalid-command', ok=False)
    finally:
        subprocess.run(base + ['kill-server'], env=env, capture_output=True, timeout=20)
print(json.dumps(results, indent=2))
