#!/usr/bin/env python3
"""Exercise runtime lifecycle, timers, streams, PTYs, and descriptor cleanup."""
import fcntl
import os
import pathlib
import pty
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time

root = pathlib.Path(__file__).resolve().parents[1]
binary = pathlib.Path(os.environ.get('HMUX_BINARY', root / 'target/debug/hmux2')).resolve()
env = dict(os.environ, TERM='xterm-256color', LC_ALL='C', TMUX='', SHELL='/bin/sh')
work = root / 'target/migration'
work.mkdir(parents=True, exist_ok=True)

def eventually(predicate, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(.03)
    raise AssertionError('condition did not become true')

with tempfile.TemporaryDirectory(prefix='runtime-', dir=work) as tmp:
    tmp = pathlib.Path(tmp)
    base = [str(binary), '-S', str(tmp / 'socket'), '-f', '/dev/null']
    def run(*args):
        return subprocess.check_output(base + list(args), env=env, timeout=15)
    attached = None
    try:
        run('new-session', '-d', '-s', 'runtime', '-x', '80', '-y', '24', '/bin/sh')
        pid = int(run('display-message', '-p', '#{pid}'))
        run('set-option', '-g', 'status-interval', '1')
        # Delayed and ordinary jobs must run and be reaped.
        started = time.monotonic()
        run('run-shell', '-d', '0.1', 'true')
        assert time.monotonic() - started >= .08
        for _ in range(4):
            run('if-shell', 'exit 0', 'set-option -g @job ok')
        assert run('show-options', '-gv', '@job') == b'ok\n'
        # Binary file transport, including enough bytes to require partial writes.
        payload = bytes(range(256)) * 4096
        (tmp / 'input').write_bytes(payload)
        run('load-buffer', '-b', 'large', str(tmp / 'input'))
        run('save-buffer', '-b', 'large', str(tmp / 'output'))
        assert (tmp / 'output').read_bytes() == payload
        # Configuration progress must also avoid repeatedly coalescing data.
        (tmp / 'large.conf').write_text('# padding for segmented file input\n' * 8192
                                        + 'set-option -g @loaded-source yes\n')
        run('source-file', str(tmp / 'large.conf'))
        assert run('show-options', '-gv', '@loaded-source') == b'yes\n'
        # Both pipe directions: producer input reaches the pane and pane output
        # reaches the pipe consumer before pipe teardown.
        piped = tmp / 'piped'
        run('pipe-pane', '-O', '-t', 'runtime:0.0', 'cat > ' + str(piped))
        run('send-keys', '-t', 'runtime:0.0', 'printf runtime-pipe-output', 'Enter')
        eventually(lambda: piped.exists() and b'runtime-pipe-output' in piped.read_bytes())
        run('pipe-pane', '-t', 'runtime:0.0')
        run('pipe-pane', '-I', '-t', 'runtime:0.0', "printf 'printf runtime-pipe-input\\n'")
        eventually(lambda: b'runtime-pipe-input' in run('capture-pane', '-p', '-t', 'runtime:0.0'))
        run('pipe-pane', '-t', 'runtime:0.0')
        # Fill a control client's pipe while it is deliberately not reading,
        # then drain and require every command completion before disconnect.
        slow = subprocess.Popen(base + ['-C', 'attach-session', '-t', 'runtime'], env=env,
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        slow.stdin.write(b'list-commands\n' * 20)
        slow.stdin.flush()
        time.sleep(.15)
        output, errors = slow.communicate(b'detach-client\n', timeout=15)
        assert slow.returncode == 0, errors
        assert len(output) > 128 * 1024 and output.count(b'%end ') >= 21
        # Attach using a real controlling PTY, resize it, pass an escape key,
        # exercise copy-mode key timers, and detach using terminal input.
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 30, 100, 0, 0))
        def controlling_tty():
            os.setsid()
            fcntl.ioctl(0, termios.TIOCSCTTY, 0)
        attached = subprocess.Popen(base + ['attach-session', '-t', 'runtime'], env=env,
                                    stdin=slave, stdout=slave, stderr=slave, preexec_fn=controlling_tty)
        os.close(slave)
        def drain():
            while select.select([master], [], [], 0)[0]:
                try:
                    os.read(master, 65536)
                except OSError:
                    break
        eventually(lambda: run('list-clients', '-F', '#{client_pid}').strip() != b'')
        fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack('HHHH', 32, 110, 0, 0))
        os.kill(attached.pid, signal.SIGWINCH)
        eventually(lambda: run('display-message', '-p', '#{window_width}x#{window_height}') == b'110x31\n')
        os.write(master, b'\x1b')
        time.sleep(.1)
        run('copy-mode', '-t', 'runtime:0.0')
        run('send-keys', '-X', '-t', 'runtime:0.0', 'cancel')
        # Drain status refresh output while the terminal is attached.
        deadline = time.monotonic() + 1.1
        while time.monotonic() < deadline:
            drain()
            time.sleep(.02)
        os.write(master, b'\x02d')
        eventually(lambda: (drain() or attached.poll() is not None))
        assert attached.returncode == 0
        os.close(master)
        time.sleep(.15)
        baseline = len(list(pathlib.Path(f'/proc/{pid}/fd').iterdir()))
        for n in range(8):
            run('new-window', '-d', '-n', f'temp{n}', 'sleep 60')
            run('kill-window', '-t', f'runtime:temp{n}')
            run('run-shell', 'true')
        eventually(lambda: len(list(pathlib.Path(f'/proc/{pid}/fd').iterdir())) == baseline)
        # There must be no unreaped children after transient jobs/windows exit.
        children = pathlib.Path(f'/proc/{pid}/task/{pid}/children').read_text().split()
        assert all(pathlib.Path(f'/proc/{child}/stat').read_text().split(') ', 1)[1][0] != 'Z' for child in children)
        print('forked startup, jobs, timers, binary files, pipes, PTY resize/detach, and fd cleanup passed')
    finally:
        subprocess.run(base + ['kill-server'], env=env, capture_output=True, timeout=10)
        if attached is not None and attached.poll() is None:
            attached.terminate()
            attached.wait(timeout=5)
    # Foreground startup exercises the non-fork server path.
    foreground = subprocess.Popen([str(binary), '-D', '-S', str(tmp / 'foreground'), '-f', '/dev/null'], env=env,
                                  stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    other = [str(binary), '-S', str(tmp / 'foreground')]
    try:
        eventually(lambda: (tmp / 'foreground').exists())
        subprocess.run(other + ['new-session', '-d', '-s', 'foreground', 'sleep 60'], env=env, check=True, timeout=10)
        assert subprocess.check_output(other + ['display-message', '-p', '#{session_name}'], env=env, timeout=10) == b'foreground\n'
        subprocess.run(other + ['kill-server'], env=env, check=True, timeout=10)
        assert foreground.wait(timeout=10) == 0
        print('non-fork startup and shutdown passed')
    finally:
        if foreground.poll() is None:
            foreground.terminate()
            foreground.wait(timeout=5)
