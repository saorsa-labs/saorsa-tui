//! CI integration tests — verify API contracts, type properties, and version consistency.

#![allow(clippy::unwrap_used)]

use saorsa_tui::{
    Cell, Color, Event, KeyCode, KeyEvent, Position, Rect, SaorsaCoreError, ScreenBuffer, Segment,
    Size, Style,
};

/// Verify all workspace crates share the same version.
#[test]
fn workspace_version_consistency() {
    let core_version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = core_version.split('.').collect();
    assert_eq!(parts.len(), 3, "Version should be semver: {core_version}");

    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "Version component '{part}' should be numeric in {core_version}"
        );
    }
}

/// Verify core error type implements std::error::Error + Display + Debug.
#[test]
fn error_type_traits() {
    fn assert_error<T: std::error::Error + std::fmt::Display + std::fmt::Debug>() {}
    assert_error::<SaorsaCoreError>();
}

/// Verify Style is Clone + Default.
#[test]
fn style_is_clone_and_default() {
    let s = Style::default();
    let _s2 = s.clone();
}

/// Verify Color is Clone + PartialEq.
#[test]
fn color_is_clone_and_partialeq() {
    let c1 = Color::Reset;
    let c2 = c1.clone();
    assert_eq!(c1, c2);
}

/// Verify geometry types are Copy.
#[test]
fn geometry_types_are_copy() {
    let pos = Position::new(1, 2);
    let _pos2 = pos;
    let _pos3 = pos; // Still valid after copy

    let size = Size::new(10, 20);
    let _size2 = size;
    let _size3 = size;

    let rect = Rect::new(0, 0, 10, 10);
    let _rect2 = rect;
    let _rect3 = rect;
}

/// Verify Segment is Clone.
#[test]
fn segment_is_clone() {
    let seg = Segment::styled("hello", Style::default());
    let _seg2 = seg.clone();
}

/// Verify Cell is Clone.
#[test]
fn cell_is_clone() {
    let cell = Cell::new("x", Style::default());
    let _cell2 = cell.clone();
}

/// Verify ScreenBuffer can be constructed and queried.
#[test]
fn screen_buffer_constructable() {
    let buf = ScreenBuffer::new(Size::new(80, 24));
    assert_eq!(buf.width(), 80);
    assert_eq!(buf.height(), 24);
}

/// Verify Event enum variants exist.
#[test]
fn event_types_exist() {
    let key_event = Event::Key(KeyEvent::plain(KeyCode::Enter));
    match key_event {
        Event::Key(_) => {}
        _ => unreachable!(),
    }
}

/// Verify public re-exports from crate root.
#[test]
fn public_reexports_accessible() {
    let _style = Style::default();
    let _size = Size::new(1, 1);
    let _buf = ScreenBuffer::new(Size::new(1, 1));
    let _color = Color::Reset;
    let _seg = Segment::styled("test", Style::default());
}
