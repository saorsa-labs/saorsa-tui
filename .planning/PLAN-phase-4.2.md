# Phase 4.2: Data Widgets

## Overview

Build specialized data display and interaction widgets: RichLog (scrollable structured log), SelectList (keyboard-navigable list with fuzzy filtering), DataTable (sortable/scrollable table with columns), Tree/DirectoryTree (expandable hierarchical view), and DiffView (side-by-side/unified diff display).

These widgets build on the Phase 4.1 text infrastructure (TextBuffer, cursor, wrapping) and compositor/layout systems (Phase 3).

**New dependencies:**
- `fuzzy-matcher = "0.3"` — fuzzy string matching for SelectList filtering
- `similar = "2.6"` — text diffing for DiffView

**Widget patterns:**
- All widgets implement `Widget` trait: `render(&self, area: Rect, buf: &mut ScreenBuffer)`
- Interactive widgets implement `InteractiveWidget` trait: `handle_event(&mut self, event: &Event) -> EventResult`
- Builder pattern: `new()` → `with_*()` methods → render
- UTF-8 safe: use `truncate_to_display_width()` and `string_display_width()` from `text.rs`
- NO `.unwrap()` or `.expect()` in production code (OK in tests with `assert!()` + `match` pattern)

---

## Task 1: RichLog Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/rich_log.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add RichLog to exports)

**Description:**
Create a scrollable log widget that displays structured log entries, each as a vector of styled Segments. Supports vertical scrolling via keyboard (Up/Down/PageUp/PageDown/Home/End) and auto-scrolling to bottom on new entries.

Public API:
```rust
pub struct RichLog {
    /// Log entries: each entry is a line of Segments
    entries: Vec<Vec<Segment>>,
    /// Index of the first visible entry
    scroll_offset: usize,
    /// Base style for the log area
    style: Style,
    /// Whether to auto-scroll to bottom when entries are added
    auto_scroll: bool,
    /// Border style (optional)
    border: Option<BorderStyle>,
}

impl RichLog {
    pub fn new() -> Self;
    pub fn with_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    pub fn with_auto_scroll(self, enabled: bool) -> Self;
    
    /// Add a log entry (single line of segments)
    pub fn push(&mut self, entry: Vec<Segment>);
    /// Add a plain text entry (convenience)
    pub fn push_text(&mut self, text: &str);
    /// Clear all entries
    pub fn clear(&mut self);
    /// Get total entry count
    pub fn len(&self) -> usize;
    /// Check if empty
    pub fn is_empty(&self) -> bool;
    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self);
    /// Scroll to top
    pub fn scroll_to_top(&mut self);
    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize;
}
```

Widget rendering:
- Calculate visible area (minus border if present)
- Render visible slice of entries: `entries[scroll_offset..scroll_offset+height]`
- Each entry line: convert Segments to Cells, UTF-8 safe truncation if too wide
- Empty space below last entry: fill with blank cells

InteractiveWidget event handling:
- Up/Down: scroll by 1 line (clamp to valid range)
- PageUp/PageDown: scroll by visible height
- Home/End: scroll to top/bottom
- Disable auto-scroll when user manually scrolls

**Tests (~12):**
- Create empty log, push entries, verify length
- Render with no entries (blank area)
- Render with entries: verify visible slice
- Render with border: verify border + inner content area
- Render with multi-segment entries: verify styling preserved
- Scroll operations: up/down/page/home/end
- Auto-scroll on push when enabled
- Manual scroll disables auto-scroll
- UTF-8 safety: wide chars and emoji in entries
- Overflow: entry wider than area is truncated safely
- Empty log keyboard events ignored gracefully

---

## Task 2: SelectList Core Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/select_list.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add SelectList to exports)

**Description:**
Create a keyboard-navigable list widget that displays items and highlights the selected one. Supports vertical scrolling, keyboard navigation (Up/Down/Home/End/PageUp/PageDown), and selection confirmation (Enter).

Public API:
```rust
pub struct SelectList<T> {
    /// List items (generic data)
    items: Vec<T>,
    /// Function to render an item as Segments
    render_fn: Box<dyn Fn(&T) -> Vec<Segment>>,
    /// Index of currently selected item
    selected: usize,
    /// Scroll offset (first visible item)
    scroll_offset: usize,
    /// Style for unselected items
    item_style: Style,
    /// Style for selected item
    selected_style: Style,
    /// Border style
    border: Option<BorderStyle>,
    /// Callback on selection confirmation (Enter pressed)
    on_select: Option<Box<dyn FnMut(&T)>>,
}

impl<T> SelectList<T> {
    pub fn new(items: Vec<T>) -> Self;
    pub fn with_render_fn<F>(self, f: F) -> Self where F: Fn(&T) -> Vec<Segment> + 'static;
    pub fn with_selected_style(self, style: Style) -> Self;
    pub fn with_item_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    pub fn with_on_select<F>(self, f: F) -> Self where F: FnMut(&T) + 'static;
    
    /// Get current items
    pub fn items(&self) -> &[T];
    /// Set items (resets selection to 0)
    pub fn set_items(&mut self, items: Vec<T>);
    /// Get selected index
    pub fn selected(&self) -> usize;
    /// Set selected index (clamped)
    pub fn set_selected(&mut self, idx: usize);
    /// Get currently selected item reference
    pub fn selected_item(&self) -> Option<&T>;
    /// Move selection up/down by n
    pub fn move_selection(&mut self, delta: isize);
}
```

Widget rendering:
- Calculate visible area
- Render items[scroll_offset..scroll_offset+height]
- Highlight selected item with selected_style
- Ensure selected item is visible (adjust scroll_offset if needed)
- UTF-8 safe truncation for long items

InteractiveWidget event handling:
- Up/Down: move selection (with auto-scroll)
- PageUp/PageDown: move by visible height
- Home/End: select first/last
- Enter: trigger on_select callback
- Return Consumed if handled, Ignored otherwise

**Tests (~15):**
- Create list with items, verify item access
- Render empty list (blank area)
- Render with items: verify all visible items
- Render with selected item: verify highlight style
- Scroll offset adjusted when selection out of view
- Keyboard navigation: up/down/page/home/end
- Selection wraps correctly at boundaries
- Enter key triggers on_select callback
- Custom render function used for item display
- Border rendering
- UTF-8 safety: wide chars in item text
- Empty list handles events gracefully
- Set items resets selection to 0
- Move selection with positive/negative delta
- Selected item reference retrieval

---

## Task 3: SelectList Fuzzy Filtering

**Files:**
- MODIFY: `crates/fae-core/src/widget/select_list.rs`
- MODIFY: `crates/fae-core/Cargo.toml` (add fuzzy-matcher)
- MODIFY: `Cargo.toml` (add fuzzy-matcher to workspace)

**Description:**
Extend SelectList with fuzzy filtering capability. User types a filter query; list displays only matching items with match highlighting.

Added fields:
```rust
pub struct SelectList<T> {
    // ... existing fields ...
    
    /// Function to extract search text from item
    search_fn: Option<Box<dyn Fn(&T) -> String>>,
    /// Current filter query
    filter_query: String,
    /// Filtered indices (maps display index -> items index)
    filtered_indices: Vec<usize>,
    /// Whether filtering is active
    filter_active: bool,
}
```

New methods:
```rust
impl<T> SelectList<T> {
    pub fn with_search_fn<F>(self, f: F) -> Self where F: Fn(&T) -> String + 'static;
    pub fn enable_filter(&mut self);
    pub fn disable_filter(&mut self);
    pub fn filter_query(&self) -> &str;
    pub fn set_filter_query(&mut self, query: &str);
    pub fn clear_filter(&mut self);
    pub fn is_filter_active(&self) -> bool;
    pub fn filtered_items(&self) -> Vec<&T>;
}
```

Filtering behavior:
- When filter active: only render items matching filter_query (fuzzy match)
- Use `fuzzy-matcher` crate's `skim` algorithm
- Update filtered_indices on query change
- Selected index refers to filtered list
- Disable filter: restore full list, preserve selection if possible

InteractiveWidget event changes:
- Char input: append to filter_query, update filter
- Backspace: remove from query, update filter
- Esc: clear filter and disable
- Existing navigation works on filtered list

**Tests (~15):**
- Enable/disable filter
- Set filter query: verify filtered_indices updated
- Fuzzy matching: "abc" matches "a_b_c", "axbxc", etc.
- Render filtered list: only matching items shown
- Selected index operates on filtered list
- Navigation on filtered list
- Clear filter restores full list
- Backspace removes filter characters
- Esc clears and disables filter
- Empty query shows all items
- No matches: empty filtered list
- Filter with custom search_fn
- UTF-8 safe query input
- Filter preserves selection if item still matches
- Char input triggers filter update and re-render

---

## Task 4: DataTable Core Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/data_table.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add DataTable to exports)

**Description:**
Create a scrollable data table widget with columns and rows. Supports keyboard navigation (arrows/page/home/end) and vertical/horizontal scrolling.

Public API:
```rust
pub struct Column {
    pub header: String,
    pub width: u16,
    pub alignment: Alignment,
}

pub struct DataTable {
    /// Column definitions
    columns: Vec<Column>,
    /// Rows: each row is Vec<String> (one per column)
    rows: Vec<Vec<String>>,
    /// Selected row index
    selected_row: usize,
    /// Scroll offset (first visible row)
    row_offset: usize,
    /// Horizontal scroll offset (first visible column)
    col_offset: u16,
    /// Style for headers
    header_style: Style,
    /// Style for unselected rows
    row_style: Style,
    /// Style for selected row
    selected_style: Style,
    /// Border style
    border: Option<BorderStyle>,
}

impl DataTable {
    pub fn new(columns: Vec<Column>) -> Self;
    pub fn with_header_style(self, style: Style) -> Self;
    pub fn with_row_style(self, style: Style) -> Self;
    pub fn with_selected_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    
    /// Add a row
    pub fn push_row(&mut self, row: Vec<String>);
    /// Set all rows at once
    pub fn set_rows(&mut self, rows: Vec<Vec<String>>);
    /// Get row count
    pub fn row_count(&self) -> usize;
    /// Get column count
    pub fn column_count(&self) -> usize;
    /// Get selected row index
    pub fn selected_row(&self) -> usize;
    /// Set selected row
    pub fn set_selected_row(&mut self, idx: usize);
    /// Get selected row data
    pub fn selected_row_data(&self) -> Option<&[String]>;
}
```

Widget rendering:
- First row: render column headers
- Following rows: render data rows (visible slice)
- Each cell: truncate to column width (UTF-8 safe), apply alignment
- Highlight selected row
- Handle horizontal scrolling (skip columns before col_offset)
- Border around entire table

InteractiveWidget event handling:
- Up/Down: move selected row (with auto-scroll)
- Left/Right: horizontal scroll
- PageUp/PageDown: scroll by visible height
- Home/End: select first/last row
- Ctrl+Left/Right: scroll to first/last column

**Tests (~15):**
- Create table with columns, verify column count
- Add rows, verify row count
- Render empty table (headers only)
- Render with rows: verify header + data rows
- Selected row highlighted
- Column alignment: left/center/right
- UTF-8 safe truncation in cells
- Vertical scrolling: row navigation
- Horizontal scrolling: column navigation
- PageUp/PageDown behavior
- Home/End navigation
- Border rendering
- Empty table with columns defined
- Cell overflow truncation
- Wide characters in cell content

---

## Task 5: DataTable Sorting & Column Resize

**Files:**
- MODIFY: `crates/fae-core/src/widget/data_table.rs`

**Description:**
Add sorting and interactive column width adjustment to DataTable.

Added fields:
```rust
pub struct DataTable {
    // ... existing fields ...
    
    /// Sort state: (column_index, ascending)
    sort_state: Option<(usize, bool)>,
    /// Whether columns can be resized
    resizable_columns: bool,
}
```

New methods:
```rust
impl DataTable {
    pub fn with_resizable_columns(self, enabled: bool) -> Self;
    
    /// Sort by column (toggle ascending/descending)
    pub fn sort_by_column(&mut self, col_idx: usize);
    /// Clear sort
    pub fn clear_sort(&mut self);
    /// Get current sort state
    pub fn sort_state(&self) -> Option<(usize, bool)>;
    /// Resize column width
    pub fn set_column_width(&mut self, col_idx: usize, width: u16);
    /// Get column width
    pub fn column_width(&self, col_idx: usize) -> Option<u16>;
}
```

Sorting behavior:
- Sort rows lexicographically by selected column
- Toggle ascending/descending on repeated sort
- Preserve selection (track row data, not index)
- Render sort indicator in header: "↑" / "↓"

Column resizing:
- Ctrl+Shift+Left/Right: decrease/increase selected column width
- Minimum column width: 3 characters
- Maximum column width: configurable (default 50)

InteractiveWidget event additions:
- Ctrl+1..9: sort by column 1-9
- Ctrl+0: clear sort
- Ctrl+Shift+Left/Right: resize current column

**Tests (~12):**
- Sort by column: verify row order
- Toggle ascending/descending
- Sort indicator in header
- Multiple sorts: verify last sort wins
- Clear sort restores original order
- Column resize: increase/decrease width
- Column resize clamping: min/max bounds
- Keyboard shortcuts for sorting
- Keyboard shortcuts for resizing
- Selection preserved after sort
- UTF-8 safe sorting
- Empty table sorting (no crash)

---

## Task 6: Tree Widget Core

**Files:**
- CREATE: `crates/fae-core/src/widget/tree.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add Tree to exports)

**Description:**
Create a hierarchical tree widget with expandable/collapsible nodes. Supports keyboard navigation and lazy loading.

Public API:
```rust
pub struct TreeNode<T> {
    pub data: T,
    pub children: Vec<TreeNode<T>>,
    pub expanded: bool,
    pub is_leaf: bool,
}

pub struct Tree<T> {
    /// Root nodes (forest)
    roots: Vec<TreeNode<T>>,
    /// Flattened visible nodes (pre-order traversal, respecting expand state)
    visible_nodes: Vec<(usize, &TreeNode<T>)>, // (depth, node)
    /// Selected visible node index
    selected: usize,
    /// Scroll offset (first visible line)
    scroll_offset: usize,
    /// Function to render a node
    render_fn: Box<dyn Fn(&T, usize, bool, bool) -> Vec<Segment>>, // (data, depth, expanded, is_leaf)
    /// Style for unselected nodes
    node_style: Style,
    /// Style for selected node
    selected_style: Style,
    /// Border style
    border: Option<BorderStyle>,
    /// Lazy load callback: fn(node) -> Vec<TreeNode<T>>
    lazy_load_fn: Option<Box<dyn Fn(&T) -> Vec<TreeNode<T>>>>,
}

impl<T> Tree<T> {
    pub fn new(roots: Vec<TreeNode<T>>) -> Self;
    pub fn with_render_fn<F>(self, f: F) -> Self where F: Fn(&T, usize, bool, bool) -> Vec<Segment> + 'static;
    pub fn with_node_style(self, style: Style) -> Self;
    pub fn with_selected_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    pub fn with_lazy_load<F>(self, f: F) -> Self where F: Fn(&T) -> Vec<TreeNode<T>> + 'static;
    
    /// Toggle expand/collapse at selected node
    pub fn toggle_selected(&mut self);
    /// Expand selected node
    pub fn expand_selected(&mut self);
    /// Collapse selected node
    pub fn collapse_selected(&mut self);
    /// Get selected visible node
    pub fn selected_node(&self) -> Option<&TreeNode<T>>;
    /// Rebuild visible_nodes list from roots
    pub fn rebuild_visible(&mut self);
}
```

Widget rendering:
- Render visible_nodes[scroll_offset..scroll_offset+height]
- Each node: indentation (2 spaces per depth level) + expand indicator ("▶" / "▼") + content
- Selected node highlighted
- UTF-8 safe truncation

InteractiveWidget event handling:
- Up/Down: navigate visible nodes
- Right: expand selected node (lazy load if needed)
- Left: collapse selected node (or move to parent if already collapsed)
- Enter: toggle expand/collapse
- PageUp/PageDown/Home/End: scroll

**Tests (~15):**
- Create tree with nodes, verify root count
- Render collapsed tree: only roots visible
- Expand node: children become visible
- Collapse node: children hidden
- Navigate visible nodes: up/down
- Right key expands, left key collapses
- Enter toggles expand/collapse
- Lazy load: expand triggers callback, children loaded
- Selected node retrieval
- Rebuild visible list after expansion change
- UTF-8 safe node labels
- Empty tree (no roots)
- Deep tree: multiple levels of depth
- Mixed expanded/collapsed states
- Border rendering

---

## Task 7: DirectoryTree Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/directory_tree.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add DirectoryTree to exports)

**Description:**
Create a filesystem directory tree widget, a concrete implementation of Tree<PathBuf> with lazy loading.

Public API:
```rust
pub struct DirectoryTree {
    /// Underlying tree widget
    tree: Tree<PathBuf>,
    /// Show hidden files
    show_hidden: bool,
}

impl DirectoryTree {
    /// Create from a root path
    pub fn new(root: PathBuf) -> Result<Self, FaeCoreError>;
    pub fn with_show_hidden(self, enabled: bool) -> Self;
    pub fn with_node_style(self, style: Style) -> Self;
    pub fn with_selected_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    
    /// Get selected path
    pub fn selected_path(&self) -> Option<&PathBuf>;
    /// Toggle expand/collapse at selection
    pub fn toggle_selected(&mut self);
}
```

Implementation details:
- Lazy load: on expand, read directory with `std::fs::read_dir`
- Render function: show filename, icons for dirs/files (e.g., "📁" / "📄")
- Sort: directories first, then files, alphabetically
- Handle errors gracefully (permission denied, etc.)
- Filter hidden files if show_hidden=false

Delegates rendering and event handling to underlying Tree<PathBuf>.

**Tests (~10):**
- Create directory tree from test fixture path
- Lazy load: expand directory, verify children loaded
- Render with file/directory icons
- Hidden files filtered when show_hidden=false
- Hidden files shown when show_hidden=true
- Selected path retrieval
- Navigate and expand nested directories
- Handle permission errors gracefully
- Empty directory expands to no children
- UTF-8 safe file names

---

## Task 8: DiffView Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/diff_view.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add DiffView to exports)
- MODIFY: `crates/fae-core/Cargo.toml` (add similar)
- MODIFY: `Cargo.toml` (add similar to workspace)

**Description:**
Create a diff viewer widget that displays text differences in side-by-side or unified format. Uses the `similar` crate for diffing.

Public API:
```rust
pub enum DiffMode {
    SideBySide,
    Unified,
}

pub struct DiffView {
    /// Original text
    old_text: String,
    /// Modified text
    new_text: String,
    /// Display mode
    mode: DiffMode,
    /// Scroll offset
    scroll_offset: usize,
    /// Style for unchanged lines
    unchanged_style: Style,
    /// Style for added lines
    added_style: Style,
    /// Style for removed lines
    removed_style: Style,
    /// Border style
    border: Option<BorderStyle>,
}

impl DiffView {
    pub fn new(old_text: &str, new_text: &str) -> Self;
    pub fn with_mode(self, mode: DiffMode) -> Self;
    pub fn with_unchanged_style(self, style: Style) -> Self;
    pub fn with_added_style(self, style: Style) -> Self;
    pub fn with_removed_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    
    /// Set texts and recompute diff
    pub fn set_texts(&mut self, old_text: &str, new_text: &str);
    /// Switch display mode
    pub fn set_mode(&mut self, mode: DiffMode);
    /// Get current mode
    pub fn mode(&self) -> DiffMode;
}
```

Widget rendering (Unified mode):
- Compute line-by-line diff using `similar::TextDiff`
- Prefix: " " (unchanged), "+" (added), "-" (removed)
- Color lines: unchanged (default), added (green bg), removed (red bg)
- Render visible slice with scrolling

Widget rendering (SideBySide mode):
- Split area vertically: left=old, right=new
- Align changes: if line removed on left, blank on right; if added on right, blank on left
- Synchronized scrolling

InteractiveWidget event handling:
- Up/Down: scroll by line
- PageUp/PageDown: scroll by page
- Home/End: scroll to top/bottom
- 'm': toggle mode (side-by-side ↔ unified)

**Tests (~12):**
- Create diff view with old/new text
- Unified mode: verify prefixes and colors
- Side-by-side mode: verify left/right split
- Scrolling: up/down/page/home/end
- Toggle mode with 'm' key
- Empty diff (identical texts)
- All added (old text empty)
- All removed (new text empty)
- Mixed changes: add, remove, unchanged
- UTF-8 safe: wide chars in diff
- Border rendering
- Compute diff on set_texts

---

## Critical Files for Implementation

- `/Users/davidirvine/Desktop/Devel/projects/fae/crates/fae-core/src/widget/rich_log.rs` — Scrollable log with styled entries
- `/Users/davidirvine/Desktop/Devel/projects/fae/crates/fae-core/src/widget/select_list.rs` — Keyboard-navigable list with fuzzy filtering
- `/Users/davidirvine/Desktop/Devel/projects/fae/crates/fae-core/src/widget/data_table.rs` — Sortable table with column resizing
- `/Users/davidirvine/Desktop/Devel/projects/fae/crates/fae-core/src/widget/tree.rs` — Core tree widget with lazy loading
- `/Users/davidirvine/Desktop/Devel/projects/fae/crates/fae-core/src/widget/diff_view.rs` — Unified and side-by-side diff rendering
