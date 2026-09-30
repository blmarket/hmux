import unittest
from pathlib import Path
import subprocess
import tempfile
from unittest.mock import patch

from model_boundary_inventory import (
    MODELS, mask, enclosing_function, owner, private_fields, field_visibility, classify,
)
from client_storage_boundary import private_storage_probe


class BoundaryInventoryTests(unittest.TestCase):
    def test_client_storage_probe_seals_projection_without_changing_consumers(self):
        source = {
            'src/shared/client.rs': 'type R = Rc<UnsafeCell<client>>; type W = Weak<UnsafeCell<client>>;',
            'src/server_client/mod.rs': 'pub use model::client; fn make() { std::cell::UnsafeCell::new(value) }',
            'src/server_client/model.rs': 'pub struct client {}',
            'src/tty.rs': 'fn callback(c: &ClientRef) { c.get(); }',
        }
        changed = private_storage_probe(source)
        self.assertIn('ClientStorage::new(value)', changed['src/server_client/mod.rs'])
        self.assertEqual(changed['src/shared/client.rs'].count('server_client::ClientStorage'), 2)
        self.assertIn('pub(super) fn get', changed['src/server_client/model.rs'])
        self.assertEqual(changed['src/tty.rs'], source['src/tty.rs'])
        self.assertNotIn('ClientStorage', source['src/server_client/model.rs'])

    def test_literals_comments_lifetimes_and_nested_functions(self):
        source = '''// fn fake() { }
fn window_pane_resize<'a>(pane: &'a Pane) {
    let text = r###"fn fake() { }"###;
    let character = '}';
    /* { /* } */ } */
    fn inner() { call(); }
    pane.field = 1;
}
fn next() { other(); }
'''
        self.assertEqual(enclosing_function(source, source.index('pane.field')), 'window_pane_resize')
        self.assertEqual(enclosing_function(source, source.index('call();')), 'inner')
        self.assertEqual(enclosing_function(source, source.index('other();')), 'next')
        self.assertEqual(len(mask(source)), len(source))
        self.assertEqual(mask(source).count('\n'), source.count('\n'))

    def test_same_file_is_not_same_model_owner(self):
        self.assertEqual(owner('src/window.rs', 'window_pane_resize'), 'window_pane')
        self.assertEqual(owner('src/window.rs', 'window_resize'), 'window')
        self.assertNotEqual(owner('src/window.rs', 'winlink_set_window'), 'window')
        self.assertIsNone(owner('src/layout/core.rs', 'layout_resize'))
        self.assertEqual(owner('src/server_client/model.rs', 'client_retain'), 'client')
        self.assertNotEqual(owner('src/control.rs', 'control_write_output'), 'client')
        self.assertNotEqual(owner('src/tty.rs', 'tty_stop_tty'), 'client')
        self.assertNotEqual(owner('src/session/alerts.rs', 'alerts_set_message'), 'client')
        self.assertEqual(owner('src/window/mod.rs', 'window_add_pane'), 'window')
        self.assertEqual(owner('src/window_pane/mod.rs', 'window_pane_key'), 'window_pane')
        self.assertNotEqual(owner('src/window_pane/api.rs', 'outer_geometry'), 'window')
        self.assertIsNone(owner('src/winlink.rs', 'winlink_set_window'))

    def test_private_probe_preserves_lines_and_other_structs(self):
        source = 'pub struct session {\n    pub id: u32,\n    pub(crate) name: CString,\n}\npub struct other { pub id: u32 }'
        changed = private_fields(source, 'session')
        self.assertNotIn('pub id: u32,', changed)
        self.assertIn('pub struct other { pub id: u32 }', changed)
        self.assertEqual(changed.count('\n'), source.count('\n'))

    def test_field_visibility_matches_only_a_real_owner_submodule(self):
        for model in ('session', 'window', 'window_pane', 'client'):
            self.assertEqual(field_visibility(model), 'pub(super)')
        for path in ('src/window/mod.rs', 'src/window.rs', 'src/shared/window.rs'):
            with self.subTest(path=path), patch.dict(MODELS, window=path):
                self.assertEqual(field_visibility('window'), '')
        with self.assertRaises(ValueError):
            private_fields('pub struct window {\n    pub id: u32,\n}', 'window', 'pub(crate)')

    def test_owner_visibility_does_not_broaden_existing_private_fields(self):
        source = 'pub struct session {\n    pub id: u32,\n    hidden: u32,\n}\n'
        changed = private_fields(source, 'session', field_visibility('session'))
        self.assertIn('    pub(super) id: u32,', changed)
        self.assertIn('    hidden: u32,', changed)
        self.assertNotIn('pub(super) hidden', changed)
        self.assertEqual(changed.count('\n'), source.count('\n'))

    def test_rustc_accepts_owner_access_but_rejects_external_alias_and_raw_access(self):
        for model in ('session', 'window', 'client'):
            module = Path(MODELS[model]).parent.name
            definition = private_fields(
                f'pub struct {model} {{\n    pub id: u32,\n}}\n', model,
                field_visibility(model),
            )
            internal = f'''
mod {module} {{
    mod model {{ {definition} }}
    pub use model::{model};
    pub fn read_owner() -> u32 {{
        let value = {model} {{ id: 7 }};
        value.id
    }}
}}
'''
            external = f'''
mod external {{
    type Alias = crate::{module}::{model};
    pub unsafe fn read_pointer(value: *const Alias) -> u32 {{ (*value).id }}
    pub fn read_reference(value: &Alias) -> u32 {{ value.id }}
}}
'''
            with self.subTest(model=model), tempfile.TemporaryDirectory() as directory:
                source = Path(directory) / 'probe.rs'
                command = ['rustc', '--edition=2021', '--crate-type=lib', '--emit=metadata',
                           str(source), '-o', str(Path(directory) / 'probe.rmeta')]
                source.write_text(internal)
                accepted = subprocess.run(command, capture_output=True, text=True)
                self.assertEqual(accepted.returncode, 0, accepted.stderr)
                source.write_text(internal + external)
                rejected = subprocess.run(command, capture_output=True, text=True)
                self.assertNotEqual(rejected.returncode, 0)
                self.assertEqual(rejected.stderr.count('error[E0616]'), 2, rejected.stderr)

    def test_compiler_types_classify_aliases_and_raw_dereferences(self):
        source = 'fn window_pane_resize() { /* 한글 */ (*parent).flags = 1; (*pane).flags = 1; }'
        def diagnostic(model, expression):
            start = source.index(expression)
            return dict(level='error', message=f'field `flags` of struct `x::{model}` is private',
                        spans=[dict(file_name='src/window.rs', byte_start=len(source[:start].encode()),
                                    line_start=1, is_primary=True)])
        violations, internal, errors = classify([
            diagnostic('window', '(*parent)'), diagnostic('window_pane', '(*pane)')],
            {'src/window.rs': source})
        self.assertEqual([v['model'] for v in violations], ['window'])
        self.assertEqual(internal['window_pane'], 1)
        self.assertEqual(errors, [])


if __name__ == '__main__':
    unittest.main()
