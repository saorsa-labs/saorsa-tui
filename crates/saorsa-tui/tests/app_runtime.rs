#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(missing_docs)]

use std::cell::Cell as StdCell;
use std::rc::Rc;

use saorsa_tui::app::{App, Dom, Leaf, StyledLeaf};
use saorsa_tui::buffer::ScreenBuffer;
use saorsa_tui::event::{Event, KeyCode, KeyEvent, Modifiers};
use saorsa_tui::geometry::Rect;
use saorsa_tui::terminal::TestBackend;
use saorsa_tui::widget::{Container, Label};

struct HookWidget {
    mounted: Rc<StdCell<usize>>,
    unmounted: Rc<StdCell<usize>>,
}

impl saorsa_tui::app::NodeWidget for HookWidget {
    fn render(&mut self, _area: Rect, _buf: &mut ScreenBuffer) {}

    fn on_mount(&mut self) {
        self.mounted.set(self.mounted.get() + 1);
    }

    fn on_unmount(&mut self) {
        self.unmounted.set(self.unmounted.get() + 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn make_app(backend: &TestBackend, dom: Dom, tcss: &str) -> App {
    App::from_tcss_string(backend, dom, tcss).unwrap()
}

#[test]
fn query_ignores_detached_nodes_and_preserves_dom_order() {
    let backend = TestBackend::new(20, 5);
    let mut dom = Dom::new();

    let root = dom.create(
        "Root",
        Box::new(Leaf::new(saorsa_tui::StaticWidget::new(vec![]))),
    );
    dom.set_root(root);
    dom.set_css_id(root, "root");

    let a = dom.create("Label", Box::new(StyledLeaf::new(Label::new("A"))));
    let b = dom.create("Label", Box::new(StyledLeaf::new(Label::new("B"))));

    let mut app = make_app(
        &backend,
        dom,
        r#"
        #root { display: flex; flex-direction: column; }
        "#,
    );

    assert!(app.query("Label").unwrap().is_empty());

    app.mount(root, a).unwrap();
    app.mount(root, b).unwrap();

    let labels = app.query("Label").unwrap();
    assert_eq!(labels, vec![a, b]);
}

#[test]
fn mount_updates_layout_rects() {
    let mut backend = TestBackend::new(10, 5);
    let mut dom = Dom::new();

    let root = dom.create(
        "Root",
        Box::new(Leaf::new(saorsa_tui::StaticWidget::new(vec![]))),
    );
    dom.set_root(root);
    dom.set_css_id(root, "root");

    let a = dom.create("Label", Box::new(StyledLeaf::new(Label::new("A"))));
    let b = dom.create("Label", Box::new(StyledLeaf::new(Label::new("B"))));

    let mut app = make_app(
        &backend,
        dom,
        r#"
        #root { display: flex; flex-direction: column; }
        Label { height: 1; }
        "#,
    );

    app.mount(root, a).unwrap();
    app.mount(root, b).unwrap();

    app.render_frame(&mut backend).unwrap();

    let ra = app.rect_of(a).unwrap();
    let rb = app.rect_of(b).unwrap();

    assert_eq!(ra.position.y, 0);
    assert_eq!(rb.position.y, 1);
}

#[test]
fn remove_subtree_unmounts_widgets() {
    let backend = TestBackend::new(20, 5);
    let mut dom = Dom::new();

    let root = dom.create(
        "Root",
        Box::new(Leaf::new(saorsa_tui::StaticWidget::new(vec![]))),
    );
    dom.set_root(root);
    dom.set_css_id(root, "root");

    let mounted = Rc::new(StdCell::new(0));
    let unmounted = Rc::new(StdCell::new(0));
    let hook = dom.create(
        "Hook",
        Box::new(HookWidget {
            mounted: mounted.clone(),
            unmounted: unmounted.clone(),
        }),
    );

    let mut app = make_app(&backend, dom, "#root { display: flex; }");

    app.mount(root, hook).unwrap();
    assert_eq!(mounted.get(), 1);

    app.remove_subtree(hook).unwrap();
    assert_eq!(unmounted.get(), 1);

    // Node no longer queryable.
    assert!(app.query("Hook").unwrap().is_empty());
}

#[test]
fn query_supports_sibling_combinators() {
    let backend = TestBackend::new(20, 5);
    let mut dom = Dom::new();

    let root = dom.create(
        "Root",
        Box::new(Leaf::new(saorsa_tui::StaticWidget::new(vec![]))),
    );
    dom.set_root(root);
    dom.set_css_id(root, "root");

    let label = dom.create("Label", Box::new(StyledLeaf::new(Label::new("L"))));
    let container = dom.create("Container", Box::new(StyledLeaf::new(Container::new())));

    let mut app = make_app(
        &backend,
        dom,
        "#root { display: flex; flex-direction: row; }",
    );

    app.mount(root, label).unwrap();
    app.mount(root, container).unwrap();

    let res = app.query("Label + Container").unwrap();
    assert_eq!(res, vec![container]);
}

#[test]
fn actions_keybinding_dispatches_app_action() {
    let mut backend = TestBackend::new(20, 5);
    let mut dom = Dom::new();

    let root = dom.create(
        "Root",
        Box::new(Leaf::new(saorsa_tui::StaticWidget::new(vec![]))),
    );
    dom.set_root(root);
    dom.set_css_id(root, "root");

    let mut app = make_app(&backend, dom, "#root { display: flex; }");

    let fired = Rc::new(StdCell::new(false));
    let fired2 = fired.clone();

    app.register_action(
        "quit",
        Box::new(move |_app: &mut App| {
            fired2.set(true);
            Ok(saorsa_tui::widget::EventResult::Consumed)
        }),
    );

    app.bind_key(KeyEvent::new(KeyCode::Char('q'), Modifiers::NONE), "quit");

    let res = app
        .handle_event(&Event::Key(KeyEvent::plain(KeyCode::Char('q'))))
        .unwrap();
    assert_eq!(res, saorsa_tui::widget::EventResult::Consumed);
    assert!(fired.get());

    // Should be safe to render after actions.
    app.render_if_needed(&mut backend).unwrap();
}
