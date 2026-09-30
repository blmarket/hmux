#!/usr/bin/env python3
"""Exercise Session groups, environment inheritance and explicit teardown.

Run after cargo build --bin hmux2. Only a temporary, isolated server is used.
Arguments and output remain bytes to test invalid names and non-UTF-8 values.
"""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=(
        Path(__file__).resolve().parents[1] / 'target/debug/hmux2'))
    binary = parser.parse_args().binary.resolve()
    binary_stat = binary.stat()
    with binary.open('rb') as stream:
        binary_hash = hashlib.file_digest(stream, 'sha256').hexdigest()
    environment = dict(os.environ, TERM='xterm-256color')
    environment.pop('TMUX', None)
    # This variable must be inherited from the test server's global environment,
    # not copied from the invoking client when each session is created.
    variable = b'HMUX_SESSION_BOUNDARY_ENV'
    environment.pop(variable.decode(), None)
    with tempfile.TemporaryDirectory(prefix='hmux-session-smoke-') as directory:
        base = [os.fsencode(binary), b'-S', os.fsencode(directory + '/server.sock'),
                b'-f', b'/dev/null']

        def invoke(*arguments, check=True):
            result = subprocess.run([*base, *(os.fsencode(arg) for arg in arguments)],
                                    env=environment, capture_output=True, timeout=10)
            if check:
                assert result.returncode == 0, (
                    f'{arguments!r}: status={result.returncode}, '
                    f'stdout={result.stdout!r}, stderr={result.stderr!r}')
            return result

        def run(*arguments):
            return invoke(*arguments).stdout.rstrip(b'\n')

        def expect(actual, expected, description):
            assert actual == expected, f'{description}: expected {expected!r}, got {actual!r}'

        def display(session, value):
            return run('display-message', '-p', '-t', session, value)

        def windows(session):
            return run('list-windows', '-t', session,
                       '-F', '#{window_index}|#{window_id}|#{window_active}').splitlines()

        try:
            alpha, first = run('new-session', '-d', '-s', 'boundary-alpha', '-n', 'first',
                               '-P', '-F', '#{session_id} #{window_id}', 'cat').split()
            beta = run('new-session', '-d', '-s', 'boundary-beta', '-t', 'boundary-alpha',
                       '-P', '-F', '#{session_id}')
            assert alpha != beta
            for session in (alpha, beta):
                expect(display(session, '#{session_grouped}|#{session_group}|'
                               '#{session_group_size}|#{session_group_list}'),
                       b'1|boundary-alpha|2|boundary-alpha,boundary-beta', 'group membership')
                expect(windows(session), [b'0|' + first + b'|1'], 'initial linked window')

            shared = run('new-window', '-d', '-t', alpha + b':7', '-n', 'shared',
                         '-P', '-F', '#{window_id}', 'cat')
            assert shared != first
            expected = [b'0|' + first + b'|1', b'7|' + shared + b'|0']
            for session in (alpha, beta):
                expect(windows(session), expected, 'new window propagation and order')
            run('select-window', '-t', beta + b':7')
            expect(display(alpha, '#{window_id}'), first, 'alpha retains its selection')
            expect(display(beta, '#{window_id}'), shared, 'beta selects independently')
            run('move-window', '-r', '-t', beta)
            # Explicit renumbering affects the target session; group window
            # creation and unlinking synchronize through their own operations.
            expect(windows(alpha), expected, 'renumber leaves the other session unchanged')
            expect(windows(beta), [b'0|' + first + b'|0', b'1|' + shared + b'|1'],
                   'renumbered beta order and selection')
            run('move-window', '-r', '-t', alpha)
            expect(windows(alpha), [b'0|' + first + b'|1', b'1|' + shared + b'|0'],
                   'renumbered alpha order and selection')
            expect(display(beta, '#{window_id}'), shared, 'beta selection survives alpha renumber')

            invalid = invoke('rename-session', '-t', beta, b'boundary-\xff', check=False)
            assert invalid.returncode != 0 and b'invalid session name' in invalid.stderr
            expect(display(beta, '#{session_name}'), b'boundary-beta', 'invalid rename leaves name intact')
            renamed = b'boundary-renamed'
            run('rename-session', '-t', beta, renamed)
            expect(display(beta, '#{session_name}'), renamed, 'session rename')
            expect(display(alpha, '#{session_group_list}'), b'boundary-alpha,' + renamed,
                   'renamed group member in insertion order')

            run('set-environment', '-g', variable, 'global')
            expect(run('show-environment', '-g', variable), variable + b'=global', 'global value')
            value_format = b'#{' + variable + b'}'
            expect(display(alpha, value_format), b'global', 'global environment fallback')
            run('set-environment', '-t', alpha, variable, b'local-\xfe')
            expect(run('show-environment', '-t', alpha, variable), variable + b'=local-\xfe',
                   'byte-preserving session environment')
            expect(display(alpha, value_format), b'local-\xfe', 'local environment override')
            expect(display(beta, value_format), b'global', 'group members keep separate environments')
            run('set-environment', '-r', '-t', alpha, variable)
            expect(run('show-environment', '-t', alpha, variable), b'-' + variable,
                   'cleared environment record')
            expect(display(alpha, value_format), b'', 'clear suppresses inheritance')
            run('set-environment', '-u', '-t', alpha, variable)
            missing = invoke('show-environment', '-t', alpha, variable, check=False)
            assert missing.returncode != 0 and b'unknown variable' in missing.stderr
            expect(display(alpha, value_format), b'global', 'unset restores inheritance')
            run('set-environment', '-gu', variable)
            expect(display(alpha, value_format), b'', 'global removal reaches alpha')
            expect(display(beta, value_format), b'', 'global removal reaches beta')

            run('unlink-window', '-k', '-t', alpha + b':0')
            for session in (alpha, beta):
                expect(windows(session), [b'1|' + shared + b'|1'], 'unlink propagates through group')
            run('kill-session', '-t', alpha)
            expect(run('list-sessions', '-F', '#{session_id}|#{session_name}|#{session_group_size}'),
                   beta + b'|' + renamed + b'|1', 'other member survives explicit destruction')
            expect(windows(beta), [b'1|' + shared + b'|1'], 'shared window survives member teardown')
            expect(display(beta, '#{session_group_list}'), renamed, 'destroyed membership removed')

            run('kill-session', '-t', beta)
            deadline = time.monotonic() + 5
            while True:
                stopped = invoke('list-sessions', check=False)
                if stopped.returncode != 0 and not stopped.stdout and (
                        b'no server running' in stopped.stderr or b'error connecting to' in stopped.stderr):
                    break
                assert time.monotonic() < deadline, f'last-session teardown: {stopped!r}'
                time.sleep(0.02)
        finally:
            invoke('kill-server', check=False)
    current_stat = binary.stat()
    assert (current_stat.st_ino, current_stat.st_size, current_stat.st_mtime_ns) == (
        binary_stat.st_ino, binary_stat.st_size, binary_stat.st_mtime_ns), 'binary changed during smoke'
    print('PASS: group membership, linked window order, selection, renumber, rename, '
          'environment inheritance, unlink and final teardown')
    print(f'Binary: {binary}\nSHA256: {binary_hash}')


if __name__ == '__main__':
    main()
