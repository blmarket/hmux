#!/usr/bin/env python3
"""Exercise command tables and server callbacks on a private, temporary socket.

HMUX_BINARY selects the before/after binary. Output deliberately excludes PIDs,
socket paths, and event timestamps so transcripts can be compared exactly.
"""
import json
import os
import pathlib
import selectors
import shlex
import signal
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
BINARY = pathlib.Path(os.environ.get('HMUX_BINARY', ROOT / 'target/debug/hmux2')).resolve()
WORK = ROOT / 'target/ffi-migration'
WORK.mkdir(parents=True, exist_ok=True)
checks = []

with tempfile.TemporaryDirectory(prefix='callbacks-', dir=WORK) as tmp:
    socket = pathlib.Path(tmp) / 'socket'
    env = dict(os.environ, TERM='xterm-256color', LC_ALL='C', TMUX='', SHELL='/bin/sh')
    base = [str(BINARY), '-S', str(socket), '-f', '/dev/null']
    control = None
    server_pid = None

    def run(*args):
        return subprocess.run(base + list(args), env=env, capture_output=True,
                              text=True, check=True, timeout=20).stdout

    def control_until(marker):
        output = b''
        deadline = time.monotonic() + 20
        with selectors.DefaultSelector() as selector:
            selector.register(control.stdout, selectors.EVENT_READ)
            while marker not in output:
                remaining = deadline - time.monotonic()
                if remaining <= 0 or not selector.select(remaining):
                    raise AssertionError(('control callback timed out', marker, output))
                chunk = os.read(control.stdout.fileno(), 65536)
                if not chunk:
                    raise AssertionError(('control client exited', marker, output))
                output += chunk
        return output

    try:
        run('new-session', '-d', '-s', 'callbacks', 'sleep 60')
        server_pid = int(run('display-message', '-p', '#{pid}'))
        commands = run('list-commands')
        for command in ['new-session', 'run-shell', 'if-shell', 'wait-for', 'kill-server']:
            assert any(line.startswith(command + ' ') for line in commands.splitlines()), command
        checks.append({'command_table_entries': len(commands.splitlines())})

        run('set-hook', '-g', 'after-new-window', 'set-option -g @hook callback-fired')
        run('new-window', '-d', '-n', 'hook', 'sleep 60')
        assert run('show-options', '-gv', '@hook') == 'callback-fired\n'
        checks.append('command-hook-dispatch')

        run('if-shell', 'exit 0', 'set-option -g @branch yes', 'set-option -g @branch no')
        assert run('show-options', '-gv', '@branch') == 'yes\n'
        run('if-shell', 'exit 1', 'set-option -g @branch yes', 'set-option -g @branch no')
        assert run('show-options', '-gv', '@branch') == 'no\n'
        checks.append('job-completion-and-command-queue-resume')

        run('run-shell', '-d', '0.01', 'exit 0', ';', 'set-option', '-g', '@timer', 'fired')
        assert run('show-options', '-gv', '@timer') == 'fired\n'
        checks.append('timer-and-child-exit-callbacks')

        producer = shlex.join(base + ['wait-for', '-S', 'background-ready'])
        run('run-shell', '-b', producer)
        run('wait-for', 'background-ready')
        checks.append('background-job-and-wait-channel')

        run('set-buffer', '-b', 'ffi', 'file-callback')
        assert run('save-buffer', '-b', 'ffi', '-') == 'file-callback'
        checks.append('client-file-write-callback')

        control = subprocess.Popen(base + ['-C', 'attach-session', '-t', 'callbacks'],
                                   env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE)
        control_until(b'%session-changed')
        control.stdin.write(b'display-message -p CONTROL-CALLBACK\n')
        control.stdin.flush()
        control_until(b'CONTROL-CALLBACK')
        run('new-window', '-d', '-n', 'notify', 'sleep 60')
        control_until(b'%window-add')
        control.stdin.write(b'detach-client\n')
        control.stdin.flush()
        control_until(b'%exit')
        assert control.wait(timeout=20) == 0
        checks.append('control-client-read-write-notify-and-detach')

        socket.unlink()
        os.kill(server_pid, signal.SIGUSR1)
        deadline = time.monotonic() + 10
        while not socket.exists():
            if time.monotonic() > deadline:
                raise AssertionError('server signal callback did not recreate its private socket')
            time.sleep(0.01)
        assert run('display-message', '-p', '#{session_name}') == 'callbacks\n'
        checks.append('server-signal-and-accept-callbacks')
    finally:
        if control is not None:
            if control.poll() is None:
                control.terminate()
                try:
                    control.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    control.kill()
                    control.wait(timeout=5)
            for stream in (control.stdin, control.stdout, control.stderr):
                stream.close()
        stopped = subprocess.run(base + ['kill-server'], env=env, capture_output=True, timeout=20)
        if stopped.returncode and server_pid is not None:
            try:
                os.kill(server_pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        deadline = time.monotonic() + 10
        while subprocess.run(base + ['list-sessions'], env=env, capture_output=True,
                             timeout=20).returncode == 0:
            if time.monotonic() > deadline:
                raise AssertionError('private server did not stop')
            time.sleep(0.01)
        # The server may leave its socket inode behind after shutting down.
        socket.unlink(missing_ok=True)
print(json.dumps(checks, indent=2))
