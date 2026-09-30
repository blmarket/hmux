import unittest
from model_boundary_inventory import mask, enclosing_function, owner, private_fields, classify
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
