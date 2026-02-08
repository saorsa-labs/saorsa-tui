#![allow(missing_docs, clippy::expect_used)]

use saorsa::app::AppState;
use saorsa::tui_runtime::SaorsaUi;
use saorsa_tui::event::{Event, KeyCode, KeyEvent, Modifiers};
use saorsa_tui::terminal::TestBackend;

#[test]
fn ctrl_r_forces_render() {
    let mut backend = TestBackend::new(40, 10);
    let mut ui = SaorsaUi::new(&backend).expect("SaorsaUi::new should succeed");

    let state = AppState::new("test-model");
    ui.sync_from_state(&state)
        .expect("sync_from_state should succeed");
    ui.render_frame(&mut backend)
        .expect("initial render should succeed");

    // With no changes, a second render should be a no-op.
    let rendered = ui
        .render_if_needed(&mut backend)
        .expect("render_if_needed should succeed");
    assert!(!rendered);

    // Manual hook: Ctrl+R marks the retained runtime dirty.
    let event = Event::Key(KeyEvent::new(KeyCode::Char('r'), Modifiers::CTRL));
    ui.handle_event(&event)
        .expect("handle_event should succeed");

    let rendered = ui
        .render_if_needed(&mut backend)
        .expect("render_if_needed should succeed");
    assert!(rendered);

    // Subsequent render should be clean again.
    let rendered = ui
        .render_if_needed(&mut backend)
        .expect("render_if_needed should succeed");
    assert!(!rendered);
}
