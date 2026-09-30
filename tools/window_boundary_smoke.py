#!/usr/bin/env python3
"""Exercise Window layout, option inheritance, hooks and explicit teardown.

Run after cargo build --bin hmux2. Only a temporary, isolated server is used.
"""
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    binary = Path(__file__).resolve().parents[1] / 'target/debug/hmux2'
    environment = dict(os.environ, TERM='xterm-256color')
    environment.pop('TMUX', None)
    with tempfile.TemporaryDirectory(prefix='hmux-window-smoke-') as directory:
        base = [str(binary), '-S', directory + '/server.sock', '-f', '/dev/null']

        def run(*arguments, check=True):
            result = subprocess.run([*base, *arguments], env=environment,
                                    capture_output=True, text=True, timeout=10, check=check)
            return result.stdout.strip()

        try:
            first, first_pane = run('new-session', '-d', '-s', 'boundary', '-x', '100',
                                    '-y', '30', '-P', '-F', '#{window_id} #{pane_id}', 'cat').split()
            second, second_pane = run('new-window', '-d', '-t', 'boundary', '-P', '-F',
                                      '#{window_id} #{pane_id}', 'cat').split()
            moving = run('split-window', '-d', '-h', '-t', first_pane, '-P', '-F',
                         '#{pane_id}', 'cat')
            run('set-option', '-w', '-t', first, '@parent', 'first')
            run('set-option', '-w', '-t', second, '@parent', 'second')
            assert run('display-message', '-p', '-t', moving, '#{@parent}') == 'first'
            run('join-pane', '-d', '-s', moving, '-t', second_pane)
            assert run('display-message', '-p', '-t', moving, '#{@parent}') == 'second'
            run('set-option', '-p', '-t', moving, '@parent', 'local')
            assert run('display-message', '-p', '-t', moving, '#{@parent}') == 'local'
            run('set-option', '-pu', '-t', moving, '@parent')
            assert run('display-message', '-p', '-t', moving, '#{@parent}') == 'second'

            run('set-option', '-w', '-t', second, 'pane-colours[0]', 'red')
            assert run('show-options', '-wv', '-t', second, 'pane-colours[0]') == 'red'
            run('set-option', '-wu', '-t', second, 'pane-colours[0]')
            assert run('show-options', '-wqv', '-t', second, 'pane-colours[0]') == ''
            run('set-hook', '-w', '-t', second, 'after-rename-window',
                'set-option -g @rename-hook called')
            run('rename-window', '-t', second, 'renamed')
            assert run('show-options', '-gqv', '@rename-hook') == 'called'

            run('resize-window', '-t', second, '-x', '100', '-y', '30')
            layouts = ('even-horizontal', 'even-vertical', 'main-horizontal',
                       'main-horizontal-mirrored', 'main-vertical',
                       'main-vertical-mirrored', 'tiled')
            # Two panes exercise the single secondary branch of main layouts;
            # three panes exercise the grouped secondary and tiled branches.
            for layout in layouts:
                run('select-layout', '-t', second, layout)
            extra = run('split-window', '-d', '-h', '-t', second_pane, '-P', '-F',
                        '#{pane_id}', 'cat')
            for layout in layouts:
                run('select-layout', '-t', second, layout)
            run('kill-pane', '-t', extra)
            run('resize-pane', '-t', moving, '-L', '3')
            run('swap-pane', '-d', '-s', moving, '-t', second_pane)
            run('swap-pane', '-d', '-s', moving, '-t', first_pane)
            assert run('display-message', '-p', '-t', moving, '#{@parent}') == 'first'
            run('swap-pane', '-d', '-s', moving, '-t', first_pane)
            assert run('display-message', '-p', '-t', moving, '#{@parent}') == 'second'
            run('rotate-window', '-t', second)
            run('resize-pane', '-Z', '-t', moving)
            assert run('display-message', '-p', '-t', moving, '#{window_zoomed_flag}') == '1'
            floating = run('new-pane', '-dA', '-t', moving, '-x', '20', '-y', '10',
                           '-P', '-F', '#{pane_id}', 'cat')
            assert run('display-message', '-p', '-t', floating, '#{pane_floating_flag}') == '1'
            run('resize-pane', '-Z', '-t', moving)
            assert run('display-message', '-p', '-t', moving, '#{window_zoomed_flag}') == '0'
            assert run('display-message', '-p', '-t', floating, '#{pane_floating_flag}') == '1'
            run('kill-pane', '-t', floating)
            assert set(run('list-panes', '-t', second, '-F', '#{pane_id}').split()) == {
                moving, second_pane}
            run('kill-pane', '-t', moving)
            run('kill-window', '-t', second)
            assert run('list-windows', '-t', 'boundary', '-F', '#{window_id}') == first
            run('kill-session', '-t', 'boundary')
            print('PASS: inheritance/reparenting, option arrays, hooks, layout, zoom, teardown')
        finally:
            run('kill-server', check=False)


if __name__ == '__main__':
    main()
