#!/usr/bin/env python3
"""Exercise Client IO, prompts, overlays, control callbacks and explicit teardown.

Run after `cargo build --bin hmux2`. Uses only its own temporary server socket.
"""
import errno
import fcntl
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time


def main():
    binary = str(Path(__file__).resolve().parents[1] / 'target/debug/hmux2')
    env = dict(os.environ, TERM='xterm-256color')
    env.pop('TMUX', None)
    with tempfile.TemporaryDirectory(prefix='hmux-client-smoke-') as directory:
        base = [binary, '-S', directory + '/server.sock', '-f', '/dev/null']
        processes = []
        terminal_output = bytearray()
        master = slave = None

        def run(*args, check=True):
            return subprocess.run([*base, *args], env=env, capture_output=True,
                                  text=True, timeout=10, check=check).stdout.strip()

        def wait_for(predicate, description):
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                if predicate():
                    return
                # Drain terminal output so the rendering checks do not block on
                # an unattended PTY. No user terminal or server is touched.
                if master is not None and select.select([master], [], [], 0.02)[0]:
                    try:
                        terminal_output.extend(os.read(master, 65536))
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
                else:
                    time.sleep(0.02)
            raise AssertionError('timed out: ' + description)

        try:
            run('new-session', '-d', '-s', 'boundary', '-x', '80', '-y', '24', 'cat')
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 80, 0, 0))

            def terminal_session():
                os.setsid()
                fcntl.ioctl(0, termios.TIOCSCTTY, 0)

            attached = subprocess.Popen([*base, 'attach-session', '-t', 'boundary'], env=env,
                                        stdin=slave, stdout=slave, stderr=slave,
                                        preexec_fn=terminal_session)
            processes.append(attached)
            os.close(slave)
            slave = None
            wait_for(lambda: bool(run('list-clients', '-F', '#{client_name}')), 'terminal attach')
            name = run('list-clients', '-F', '#{client_name}')
            os.write(master, b'client-input-marker\r')
            wait_for(lambda: 'client-input-marker' in run('capture-pane', '-p', '-t', 'boundary'),
                     'terminal key input')
            assert 'Terminal 0:' in run('show-messages', '-T', '-t', name)
            run('command-prompt', '-b', '-t', name, '-p', 'boundary', 'set-option -g @answer %%')
            os.write(master, b'borrowed-briefly\r')
            wait_for(lambda: run('show-option', '-gqv', '@answer') == 'borrowed-briefly',
                     'prompt completion callback')
            terminal_output.clear()
            menu = subprocess.Popen([*base, 'display-menu', '-t', name, '-x', '0', '-y', '0',
                                     'choose', 'x', 'set-option -g @overlay closed'], env=env,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            processes.append(menu)
            wait_for(lambda: b'choose' in terminal_output, 'menu rendering')
            os.write(master, b'x')
            wait_for(lambda: run('show-option', '-gqv', '@overlay') == 'closed',
                     'overlay callback and close')
            menu.communicate(timeout=10)
            assert menu.returncode == 0
            fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack('HHHH', 30, 100, 0, 0))
            attached.send_signal(signal.SIGWINCH)
            wait_for(lambda: run('display-message', '-p', '-c', name,
                                 '#{client_width}x#{client_height}') == '100x30', 'terminal resize')
            control = subprocess.Popen([*base, '-C', 'attach-session', '-t', 'boundary'], env=env,
                                       stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE)
            processes.append(control)
            control.stdin.write(b'display-message -p "control-#{client_control_mode}"\n')
            control.stdin.flush()
            output = bytearray()

            def control_reply(expected=b'control-1\n'):
                if select.select([control.stdout], [], [], 0.05)[0]:
                    output.extend(os.read(control.stdout.fileno(), 65536))
                return expected in output

            try:
                wait_for(control_reply, 'control protocol reply')
            except AssertionError as error:
                raise AssertionError(f'{error}; received {bytes(output)!r}; exit={control.poll()}') from error

            # Exercise every monitor target through the live control protocol.
            # Each timer phase reads its current model and releases that loan
            # before reporting a change back through this same Client.
            window, pane = run('display-message', '-p', '-t', 'boundary',
                               '#{window_id} #{pane_id}').split()
            subscriptions = {'session-watch': '', 'window-watch': window,
                             'pane-watch': pane, 'windows-watch': '@*', 'panes-watch': '%*'}
            run('set-option', '-g', '@boundary-monitor', 'first')
            for subscription, target in subscriptions.items():
                command = f"refresh-client -B '{subscription}:{target}:#{{@boundary-monitor}}'\n"
                control.stdin.write(command.encode())
            control.stdin.flush()

            def subscription_reply(value, names):
                control_reply()
                lines = output.splitlines()
                return all(any(line.startswith(b'%subscription-changed ' + name.encode() + b' ')
                               and line.endswith(b' : ' + value.encode()) for line in lines)
                           for name in names)

            wait_for(lambda: subscription_reply('first', subscriptions), 'initial subscriptions')
            output.clear()
            run('set-option', '-g', '@boundary-monitor', 'second')
            wait_for(lambda: subscription_reply('second', subscriptions), 'subscription updates')
            control.stdin.write(b'refresh-client -B session-watch\n'
                                b'display-message -p subscription-removed\n')
            control.stdin.flush()
            wait_for(lambda: control_reply(b'subscription-removed\n'),
                     'subscription removal')
            output.clear()
            run('set-option', '-g', '@boundary-monitor', 'third')
            remaining = [name for name in subscriptions if name != 'session-watch']
            wait_for(lambda: subscription_reply('third', remaining), 'remaining subscriptions')
            assert b'%subscription-changed session-watch ' not in output
            # Detach with the other subscriptions still active to exercise the
            # explicit monitor cancellation in Client control teardown.
            control.stdin.write(b'detach-client\n')
            control.stdin.flush()
            control.communicate(timeout=10)
            assert control.returncode == 0
            run('detach-client', '-t', name)
            attached.wait(timeout=10)
            assert attached.returncode == 0
            assert run('list-clients', '-F', '#{client_name}') == ''
            print('PASS: terminal input, prompt, overlay, resize, terminal registry, '
                  'control subscriptions, detach')
        finally:
            run('kill-server', check=False)
            for process in processes:
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
            for descriptor in (master, slave):
                if descriptor is not None:
                    os.close(descriptor)


if __name__ == '__main__':
    main()
