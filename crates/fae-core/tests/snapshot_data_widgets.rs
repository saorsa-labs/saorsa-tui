//! Snapshot tests for data display widgets.

#[path = "snapshot_helpers.rs"]
mod snapshot_helpers;

use fae_core::buffer::ScreenBuffer;
use fae_core::geometry::{Rect, Size};
use fae_core::segment::Segment;
use fae_core::style::Style;
use fae_core::widget::{Column, DataTable, DiffView, RichLog, SelectList, Tree, TreeNode, Widget};

/// Render a widget to a text grid for snapshot testing.
fn render_widget_to_text(widget: &dyn Widget, width: u16, height: u16) -> String {
    let mut buf = ScreenBuffer::new(Size::new(width, height));
    widget.render(Rect::new(0, 0, width, height), &mut buf);

    let mut result = String::new();
    for y in 0..height {
        for x in 0..width {
            match buf.get(x, y) {
                Some(cell) => {
                    if cell.is_blank() {
                        result.push(' ');
                    } else {
                        result.push_str(&cell.grapheme);
                    }
                }
                None => result.push(' '),
            }
        }
        result.push('\n');
    }

    result
}

// --- RichLog Snapshots ---

#[test]
fn snapshot_richlog_empty() {
    let log = RichLog::new();
    let rendered = render_widget_to_text(&log, 40, 5);
    insta::assert_snapshot!("richlog_empty", rendered);
}

#[test]
fn snapshot_richlog_with_entries() {
    let mut log = RichLog::new();
    log.push_text("First log entry");
    log.push_text("Second log entry");
    log.push_text("Third log entry");
    let rendered = render_widget_to_text(&log, 40, 5);
    insta::assert_snapshot!("richlog_with_entries", rendered);
}

#[test]
fn snapshot_richlog_styled_entries() {
    let mut log = RichLog::new();
    let bold_style = Style::new().bold(true);
    log.push(vec![
        Segment::styled("ERROR:", bold_style),
        Segment::new(" Something went wrong"),
    ]);
    log.push(vec![Segment::new("INFO: Normal message")]);
    let rendered = render_widget_to_text(&log, 50, 4);
    insta::assert_snapshot!("richlog_styled_entries", rendered);
}

// --- SelectList Snapshots ---

#[test]
fn snapshot_selectlist_items() {
    let items = vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ];
    let list = SelectList::new(items).with_render_fn(|s| vec![Segment::new(s)]);
    let rendered = render_widget_to_text(&list, 30, 5);
    insta::assert_snapshot!("selectlist_items", rendered);
}

#[test]
fn snapshot_selectlist_with_selection() {
    let items = vec![
        "Option 1".to_string(),
        "Option 2".to_string(),
        "Option 3".to_string(),
    ];
    let mut list = SelectList::new(items).with_render_fn(|s| vec![Segment::new(s)]);
    list.set_selected(1);
    let rendered = render_widget_to_text(&list, 30, 5);
    insta::assert_snapshot!("selectlist_with_selection", rendered);
}

// --- DataTable Snapshots ---

#[test]
fn snapshot_datatable_basic() {
    let cols = vec![
        Column::new("Name", 15),
        Column::new("Age", 5),
        Column::new("City", 12),
    ];
    let mut table = DataTable::new(cols);
    table.push_row(vec!["Alice".into(), "30".into(), "New York".into()]);
    table.push_row(vec!["Bob".into(), "25".into(), "London".into()]);
    table.push_row(vec!["Charlie".into(), "35".into(), "Tokyo".into()]);
    let rendered = render_widget_to_text(&table, 40, 6);
    insta::assert_snapshot!("datatable_basic", rendered);
}

#[test]
fn snapshot_datatable_empty() {
    let cols = vec![Column::new("ID", 10), Column::new("Status", 15)];
    let table = DataTable::new(cols);
    let rendered = render_widget_to_text(&table, 30, 4);
    insta::assert_snapshot!("datatable_empty", rendered);
}

#[test]
fn snapshot_datatable_with_selection() {
    let cols = vec![Column::new("Item", 20), Column::new("Count", 8)];
    let mut table = DataTable::new(cols);
    table.push_row(vec!["Widget A".into(), "42".into()]);
    table.push_row(vec!["Widget B".into(), "17".into()]);
    table.push_row(vec!["Widget C".into(), "93".into()]);
    table.set_selected_row(1);
    let rendered = render_widget_to_text(&table, 35, 6);
    insta::assert_snapshot!("datatable_with_selection", rendered);
}

// --- Tree Snapshots ---

#[test]
fn snapshot_tree_collapsed() {
    let tree_node = TreeNode::branch("root".to_string())
        .with_child(TreeNode::new("child1".to_string()))
        .with_child(TreeNode::new("child2".to_string()));

    let tree = Tree::new(vec![tree_node])
        .with_render_fn(|data: &String, _depth, _expanded, _is_leaf| vec![Segment::new(data)]);

    let rendered = render_widget_to_text(&tree, 30, 5);
    insta::assert_snapshot!("tree_collapsed", rendered);
}

#[test]
fn snapshot_tree_expanded() {
    let tree_node = TreeNode::branch("root".to_string())
        .with_child(TreeNode::new("child1".to_string()))
        .with_child(TreeNode::new("child2".to_string()));

    let mut tree = Tree::new(vec![tree_node])
        .with_render_fn(|data: &String, _depth, _expanded, _is_leaf| vec![Segment::new(data)]);

    tree.toggle_selected();
    let rendered = render_widget_to_text(&tree, 30, 5);
    insta::assert_snapshot!("tree_expanded", rendered);
}

#[test]
fn snapshot_tree_nested() {
    use fae_core::event::{Event, KeyCode, KeyEvent, Modifiers};
    use fae_core::widget::InteractiveWidget;

    let tree_node = TreeNode::branch("root".to_string()).with_child(
        TreeNode::branch("branch".to_string())
            .with_child(TreeNode::new("leaf1".to_string()))
            .with_child(TreeNode::new("leaf2".to_string())),
    );

    let mut tree = Tree::new(vec![tree_node])
        .with_render_fn(|data: &String, _depth, _expanded, _is_leaf| vec![Segment::new(data)]);

    tree.expand_selected();
    let down = Event::Key(KeyEvent {
        code: KeyCode::Down,
        modifiers: Modifiers::NONE,
    });
    tree.handle_event(&down);
    tree.expand_selected();

    let rendered = render_widget_to_text(&tree, 30, 6);
    insta::assert_snapshot!("tree_nested", rendered);
}

// --- DiffView Snapshots ---

#[test]
fn snapshot_diffview_unified() {
    let old = "line1\nline2\nline3\n";
    let new = "line1\nmodified\nline3\n";
    let diff = DiffView::new(old, new);
    let rendered = render_widget_to_text(&diff, 40, 5);
    insta::assert_snapshot!("diffview_unified", rendered);
}

#[test]
fn snapshot_diffview_additions() {
    let old = "first\n";
    let new = "first\nsecond\nthird\n";
    let diff = DiffView::new(old, new);
    let rendered = render_widget_to_text(&diff, 40, 5);
    insta::assert_snapshot!("diffview_additions", rendered);
}

#[test]
fn snapshot_diffview_deletions() {
    let old = "first\nsecond\nthird\n";
    let new = "first\n";
    let diff = DiffView::new(old, new);
    let rendered = render_widget_to_text(&diff, 40, 5);
    insta::assert_snapshot!("diffview_deletions", rendered);
}
