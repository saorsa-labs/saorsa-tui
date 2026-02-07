# Phase 4.3: UI Widgets

## Overview

Build interactive UI components for user interfaces: Tabs (tabbed content switcher), ProgressBar (determinate + indeterminate), LoadingIndicator (animated spinner/dots), Collapsible (expandable sections), form controls (Switch, RadioButton, Checkbox), OptionList (simplified SelectList variant), and Sparkline (inline mini-chart).

**Note:** Modal and Toast widgets already exist from Phase 3.4 and are sufficient for current needs. Tooltip also exists. This phase focuses on the remaining UI widgets from the roadmap.

**Widget patterns (from Phase 4.1/4.2):**
- All widgets implement `Widget` trait: `render(&self, area: Rect, buf: &mut ScreenBuffer)`
- Interactive widgets implement `InteractiveWidget` trait: `handle_event(&mut self, event: &Event) -> EventResult`
- Builder pattern: `new()` → `with_*()` methods marked `#[must_use]`
- UTF-8 safe: use `truncate_to_display_width()` and `string_display_width()` from `text.rs`
- NO `.unwrap()` or `.expect()` in production code (OK in tests with `#[allow(clippy::unwrap_used)]`)

---

## Task 1: Tabs Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/tabs.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add Tabs to exports)

**Description:**
Create a tabbed content switcher widget with keyboard navigation (arrow keys, numbers) and mouse support (future).

Public API:
```rust
pub struct Tab {
    /// Tab label text.
    pub label: String,
    /// Tab content as Segments per line.
    pub content: Vec<Vec<Segment>>,
    /// Whether this tab is closable.
    pub closable: bool,
}

pub struct Tabs {
    /// Tab definitions.
    tabs: Vec<Tab>,
    /// Active tab index.
    active_tab: usize,
    /// Tab bar style.
    tab_bar_style: Style,
    /// Active tab style.
    active_tab_style: Style,
    /// Inactive tab style.
    inactive_tab_style: Style,
    /// Content style.
    content_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Tab bar position (Top or Bottom).
    tab_bar_position: TabBarPosition,
}

pub enum TabBarPosition {
    Top,
    Bottom,
}

impl Tabs {
    pub fn new(tabs: Vec<Tab>) -> Self;
    pub fn with_tab_bar_style(self, style: Style) -> Self;
    pub fn with_active_tab_style(self, style: Style) -> Self;
    pub fn with_inactive_tab_style(self, style: Style) -> Self;
    pub fn with_content_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    pub fn with_tab_bar_position(self, pos: TabBarPosition) -> Self;
    
    /// Add a tab.
    pub fn add_tab(&mut self, tab: Tab);
    /// Get active tab index.
    pub fn active_tab(&self) -> usize;
    /// Set active tab (clamped).
    pub fn set_active_tab(&mut self, idx: usize);
    /// Get active tab content.
    pub fn active_content(&self) -> Option<&[Vec<Segment>]>;
    /// Close tab at index (if closable).
    pub fn close_tab(&mut self, idx: usize);
    /// Get tab count.
    pub fn tab_count(&self) -> usize;
}
```

Widget rendering:
- Tab bar: render all tab labels horizontally (active highlighted)
- Content area: render active tab's content
- If tab bar at top: tab bar first row, content below
- If tab bar at bottom: content first, tab bar last row
- UTF-8 safe truncation for long tab labels
- Closable tabs show "×" indicator

InteractiveWidget event handling:
- Left/Right: switch tabs (wraps)
- Ctrl+1..9: jump to tab 1-9
- Ctrl+W: close active tab (if closable)
- Tab: next tab, Shift+Tab: previous tab

**Tests (~12):**
- Create tabs with multiple entries
- Render with tab bar at top
- Render with tab bar at bottom
- Active tab highlighted
- Switch tabs with Left/Right
- Jump to tab with Ctrl+1..9
- Close closable tab with Ctrl+W
- Non-closable tab ignores close
- Empty tabs list
- Single tab
- UTF-8 safe tab labels
- Border rendering

---

## Task 2: ProgressBar Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/progress_bar.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add ProgressBar to exports)

**Description:**
Create a progress bar widget supporting both determinate (0-100%) and indeterminate (animated) modes.

Public API:
```rust
pub enum ProgressMode {
    Determinate(f32), // 0.0-1.0
    Indeterminate { phase: usize }, // animation phase
}

pub struct ProgressBar {
    /// Current progress mode.
    mode: ProgressMode,
    /// Progress bar width.
    width: u16,
    /// Style for completed portion.
    filled_style: Style,
    /// Style for uncompleted portion.
    empty_style: Style,
    /// Style for the percentage label.
    label_style: Style,
    /// Show percentage text.
    show_percentage: bool,
    /// Border style.
    border: BorderStyle,
}

impl ProgressBar {
    /// Create a determinate progress bar (0.0-1.0).
    pub fn new(progress: f32) -> Self;
    /// Create an indeterminate progress bar.
    pub fn indeterminate() -> Self;
    pub fn with_width(self, width: u16) -> Self;
    pub fn with_filled_style(self, style: Style) -> Self;
    pub fn with_empty_style(self, style: Style) -> Self;
    pub fn with_label_style(self, style: Style) -> Self;
    pub fn with_show_percentage(self, show: bool) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    
    /// Set progress (0.0-1.0).
    pub fn set_progress(&mut self, progress: f32);
    /// Get current progress (None if indeterminate).
    pub fn progress(&self) -> Option<f32>;
    /// Advance indeterminate animation phase.
    pub fn tick(&mut self);
}
```

Widget rendering (determinate):
- Filled portion: `█` characters * (progress * width)
- Empty portion: `░` characters
- Percentage overlay: centered on bar (if show_percentage)

Widget rendering (indeterminate):
- Animated wave pattern: `▏▎▍▌▋▊▉█` moving left-to-right
- Phase increments on each tick

**Tests (~10):**
- Create determinate at 0%, 50%, 100%
- Render determinate: verify filled/empty chars
- Percentage label displayed correctly
- Set progress updates rendering
- Indeterminate mode creates wave
- Tick advances indeterminate animation
- Width respected
- Border rendering
- UTF-8 safe animation characters
- Progress clamped to 0.0-1.0

---

## Task 3: LoadingIndicator Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/loading_indicator.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add LoadingIndicator to exports)

**Description:**
Create animated loading indicators (spinners, dots, etc.) for async operations.

Public API:
```rust
pub enum IndicatorStyle {
    Spinner,   // ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
    Dots,      // ⠁⠂⠄⡀⢀⠠⠐⠈
    Line,      // ─\|/─\|/
    Box,       // ▖▘▝▗
    Circle,    // ◐◓◑◒
}

pub struct LoadingIndicator {
    /// Indicator style.
    style: IndicatorStyle,
    /// Current animation frame.
    frame: usize,
    /// Style for the indicator.
    indicator_style: Style,
    /// Optional message to display next to indicator.
    message: Option<String>,
}

impl LoadingIndicator {
    pub fn new() -> Self;
    pub fn with_style(self, style: IndicatorStyle) -> Self;
    pub fn with_indicator_style(self, style: Style) -> Self;
    pub fn with_message(self, message: &str) -> Self;
    
    /// Advance animation frame.
    pub fn tick(&mut self);
    /// Reset to first frame.
    pub fn reset(&mut self);
}
```

Widget rendering:
- Render current frame character(s) from style sequence
- If message present: render indicator + space + message
- Single line height

Frame sequences:
- Spinner: `["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]`
- Dots: `["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"]`
- Line: `["─", "\\", "|", "/"]`
- Box: `["▖", "▘", "▝", "▗"]`
- Circle: `["◐", "◓", "◑", "◒"]`

**Tests (~10):**
- Create with each indicator style
- Render at different frames
- Tick advances frame
- Frame wraps at end
- Reset returns to frame 0
- Message displayed with indicator
- No message: indicator only
- UTF-8 safe indicator chars
- All frame sequences valid
- Style applied correctly

---

## Task 4: Collapsible Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/collapsible.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add Collapsible to exports)

**Description:**
Create a collapsible section widget with expandable/collapsible content.

Public API:
```rust
pub struct Collapsible {
    /// Section title.
    title: String,
    /// Content lines (only visible when expanded).
    content: Vec<Vec<Segment>>,
    /// Whether expanded.
    expanded: bool,
    /// Style for title.
    title_style: Style,
    /// Style for content.
    content_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Expand indicator characters: (collapsed, expanded).
    indicators: (&'static str, &'static str),
}

impl Collapsible {
    pub fn new(title: &str) -> Self;
    pub fn with_content(self, content: Vec<Vec<Segment>>) -> Self;
    pub fn with_expanded(self, expanded: bool) -> Self;
    pub fn with_title_style(self, style: Style) -> Self;
    pub fn with_content_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    pub fn with_indicators(self, collapsed: &'static str, expanded: &'static str) -> Self;
    
    /// Toggle expanded/collapsed.
    pub fn toggle(&mut self);
    /// Set expanded state.
    pub fn set_expanded(&mut self, expanded: bool);
    /// Check if expanded.
    pub fn is_expanded(&self) -> bool;
}
```

Widget rendering:
- First line: indicator + title
- If expanded: render content lines below
- If collapsed: only render title line
- Default indicators: "▶" (collapsed), "▼" (expanded)

InteractiveWidget event handling:
- Enter/Space: toggle expand/collapse
- Left: collapse, Right: expand

**Tests (~10):**
- Create collapsed, create expanded
- Render collapsed: title only
- Render expanded: title + content
- Toggle changes state
- Set expanded explicitly
- Custom indicators
- UTF-8 safe title and content
- Border rendering
- Empty content when expanded
- Multi-line content rendering

---

## Task 5: Form Controls — Switch, RadioButton, Checkbox

**Files:**
- CREATE: `crates/fae-core/src/widget/form_controls.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add Switch, RadioButton, Checkbox to exports)

**Description:**
Create three form control widgets: Switch (toggle), RadioButton (single selection in group), and Checkbox (boolean toggle).

Public API:
```rust
// Switch (toggle on/off)
pub struct Switch {
    /// Current state.
    state: bool,
    /// Label text.
    label: String,
    /// Style when on.
    on_style: Style,
    /// Style when off.
    off_style: Style,
    /// On indicator: e.g., "[ON ]"
    on_indicator: String,
    /// Off indicator: e.g., "[OFF]"
    off_indicator: String,
}

impl Switch {
    pub fn new(label: &str) -> Self;
    pub fn with_state(self, state: bool) -> Self;
    pub fn with_on_style(self, style: Style) -> Self;
    pub fn with_off_style(self, style: Style) -> Self;
    pub fn with_indicators(self, on: &str, off: &str) -> Self;
    
    pub fn toggle(&mut self);
    pub fn set_state(&mut self, state: bool);
    pub fn state(&self) -> bool;
}

// RadioButton (single selection in group)
pub struct RadioButton {
    /// Label text.
    label: String,
    /// Whether selected.
    selected: bool,
    /// Style when selected.
    selected_style: Style,
    /// Style when unselected.
    unselected_style: Style,
}

impl RadioButton {
    pub fn new(label: &str) -> Self;
    pub fn with_selected(self, selected: bool) -> Self;
    pub fn with_selected_style(self, style: Style) -> Self;
    pub fn with_unselected_style(self, style: Style) -> Self;
    
    pub fn select(&mut self);
    pub fn deselect(&mut self);
    pub fn is_selected(&self) -> bool;
}

// Checkbox (boolean toggle with label)
pub struct Checkbox {
    /// Label text.
    label: String,
    /// Whether checked.
    checked: bool,
    /// Style when checked.
    checked_style: Style,
    /// Style when unchecked.
    unchecked_style: Style,
}

impl Checkbox {
    pub fn new(label: &str) -> Self;
    pub fn with_checked(self, checked: bool) -> Self;
    pub fn with_checked_style(self, style: Style) -> Self;
    pub fn with_unchecked_style(self, style: Style) -> Self;
    
    pub fn toggle(&mut self);
    pub fn set_checked(&mut self, checked: bool);
    pub fn is_checked(&self) -> bool;
}
```

Widget rendering:
- Switch: `[ON ] label` or `[OFF] label`
- RadioButton: `(●) label` (selected) or `( ) label` (unselected)
- Checkbox: `[✓] label` (checked) or `[ ] label` (unchecked)

InteractiveWidget event handling (all three):
- Space/Enter: toggle state (Switch, Checkbox) or select (RadioButton)

**Tests (~15):**
- Switch: create, toggle, set state, render on/off
- Switch: custom indicators
- RadioButton: create, select/deselect, render selected/unselected
- Checkbox: create, toggle, set checked, render checked/unchecked
- Styles applied correctly for each state
- UTF-8 safe labels
- Default indicators correct
- Space/Enter events toggle state
- All three widgets single-line rendering

---

## Task 6: OptionList Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/option_list.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add OptionList to exports)

**Description:**
Create a simplified variant of SelectList for static option sets (no filtering, simpler API). Useful for settings screens and forms.

Public API:
```rust
pub struct OptionList {
    /// Option labels.
    options: Vec<String>,
    /// Selected option index.
    selected: usize,
    /// Scroll offset.
    scroll_offset: usize,
    /// Unselected option style.
    option_style: Style,
    /// Selected option style.
    selected_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Optional prefix for each option.
    prefix: Option<String>,
}

impl OptionList {
    pub fn new(options: Vec<String>) -> Self;
    pub fn with_selected_style(self, style: Style) -> Self;
    pub fn with_option_style(self, style: Style) -> Self;
    pub fn with_border(self, border: BorderStyle) -> Self;
    pub fn with_prefix(self, prefix: &str) -> Self;
    
    /// Get options.
    pub fn options(&self) -> &[String];
    /// Set options (resets selection to 0).
    pub fn set_options(&mut self, options: Vec<String>);
    /// Get selected index.
    pub fn selected(&self) -> usize;
    /// Set selected index (clamped).
    pub fn set_selected(&mut self, idx: usize);
    /// Get selected option text.
    pub fn selected_option(&self) -> Option<&str>;
}
```

Widget rendering:
- Render each option as a line
- Selected option highlighted
- Optional prefix (e.g., "• ", "> ")
- UTF-8 safe truncation

InteractiveWidget event handling:
- Up/Down: move selection
- Home/End: select first/last
- PageUp/PageDown: move by page
- Enter: confirm selection (Consumed event)

**Tests (~12):**
- Create with options
- Render with selected option highlighted
- Navigate up/down
- Home/End navigation
- Set selected index
- Selected option retrieval
- Empty options list
- Single option
- Custom prefix rendering
- UTF-8 safe option text
- Border rendering
- Scroll offset adjusts for visibility

---

## Task 7: Sparkline Widget

**Files:**
- CREATE: `crates/fae-core/src/widget/sparkline.rs`
- MODIFY: `crates/fae-core/src/widget/mod.rs` (add module + export)
- MODIFY: `crates/fae-core/src/lib.rs` (add Sparkline to exports)

**Description:**
Create a mini inline chart widget for visualizing small data series (e.g., CPU usage, memory, trends).

Public API:
```rust
pub enum SparklineStyle {
    /// Block characters: ▁▂▃▄▅▆▇█
    Bars,
    /// Line chart with dots: ⠁⠂⠃⠄⠅⠆⠇⡀⡁...
    Line,
    /// Simple dots: •
    Dots,
}

pub struct Sparkline {
    /// Data points (will be scaled to fit height).
    data: Vec<f32>,
    /// Maximum data points to display.
    max_width: usize,
    /// Chart height (in lines).
    height: u16,
    /// Sparkline style.
    style: SparklineStyle,
    /// Style for the sparkline.
    chart_style: Style,
}

impl Sparkline {
    pub fn new(data: Vec<f32>) -> Self;
    pub fn with_max_width(self, width: usize) -> Self;
    pub fn with_height(self, height: u16) -> Self;
    pub fn with_style(self, style: SparklineStyle) -> Self;
    pub fn with_chart_style(self, style: Style) -> Self;
    
    /// Add a data point (oldest dropped if exceeds max_width).
    pub fn push(&mut self, value: f32);
    /// Set all data points.
    pub fn set_data(&mut self, data: Vec<f32>);
    /// Get current data.
    pub fn data(&self) -> &[f32];
    /// Clear all data.
    pub fn clear(&mut self);
}
```

Widget rendering (Bars style):
- Single line: use block chars `▁▂▃▄▅▆▇█` to represent value
- Scale data to 0-7 range (8 block levels)
- Width = min(data.len(), max_width)

Widget rendering (Line style, height > 1):
- Multi-line: draw line chart using Braille characters
- Scale data to fit height

Widget rendering (Dots style):
- Single line: `•` for each data point

**Tests (~12):**
- Create with data
- Render bars style: verify block chars
- Render line style: multi-line
- Render dots style: single line
- Push data point: newest added, oldest dropped if overflow
- Set data replaces all
- Clear removes all data
- Empty data renders blank
- Scaling: data scaled to fit chart
- Max width respected
- UTF-8 safe chart characters
- Custom chart style applied

---

## Task 8: Integration Tests & Polish

**Files:**
- MODIFY: `crates/fae-core/src/widget/mod.rs` (integration tests)
- MODIFY: `crates/fae-core/src/widget/tabs.rs` (polish)
- MODIFY: `crates/fae-core/src/widget/progress_bar.rs` (polish)
- MODIFY: `crates/fae-core/src/widget/loading_indicator.rs` (polish)
- MODIFY: `crates/fae-core/src/widget/collapsible.rs` (polish)
- MODIFY: `crates/fae-core/src/widget/form_controls.rs` (polish)
- MODIFY: `crates/fae-core/src/widget/option_list.rs` (polish)
- MODIFY: `crates/fae-core/src/widget/sparkline.rs` (polish)

**Description:**
Add integration tests in `widget/mod.rs` that exercise all Phase 4.3 widgets together, verify they work with existing infrastructure (ScreenBuffer, overlays, events), and polish edge cases.

Integration test scenarios:
1. **Multi-widget composition**: Render Tabs containing ProgressBar and LoadingIndicator
2. **Form controls group**: Multiple RadioButtons in a group (only one selected)
3. **Animated widgets**: Tick ProgressBar and LoadingIndicator, verify animation
4. **Collapsible with nested content**: Collapsible containing OptionList
5. **Sparkline with live data**: Push data points, verify oldest dropped
6. **Event propagation**: Tabs switch triggers nested widget updates
7. **Boundary conditions**: Empty data, single item, overflow handling
8. **UTF-8 safety**: All widgets with wide chars and emoji
9. **Styling**: All widgets respect custom styles
10. **Overlay integration**: Tabs used in Modal overlay

Polish checklist:
- All builder methods return `Self`
- All public methods have doc comments
- All edge cases handled gracefully (empty data, zero size, etc.)
- No `unwrap()` or `expect()` in production code
- Consistent naming conventions with existing widgets
- Error handling via Result types where needed

**Tests (~15):**
- Multi-widget rendering test
- Form controls group selection test
- Animation tick tests (progress + loading)
- Nested widget event handling
- Sparkline data overflow test
- Empty widget rendering tests
- UTF-8 safety across all widgets
- Overlay integration test
- Style propagation test
- Border rendering consistency
- Event consumption tests
- Boundary condition tests
- Zero-size area handling
- Widget state persistence
- Documentation coverage check

---

## Summary

| Task | Name | Files | Est. Tests |
|------|------|-------|-----------|
| 1 | Tabs Widget | tabs.rs | ~12 |
| 2 | ProgressBar Widget | progress_bar.rs | ~10 |
| 3 | LoadingIndicator Widget | loading_indicator.rs | ~10 |
| 4 | Collapsible Widget | collapsible.rs | ~10 |
| 5 | Form Controls | form_controls.rs | ~15 |
| 6 | OptionList Widget | option_list.rs | ~12 |
| 7 | Sparkline Widget | sparkline.rs | ~12 |
| 8 | Integration & Polish | all files above + mod.rs | ~15 |
| **Total** | | **8 new files, 2 modified** | **~96** |

**Notes:**
- Modal, Toast, and Tooltip widgets already exist from Phase 3.4 — NOT duplicated here
- All widgets follow Phase 4.1/4.2 patterns: builder API, UTF-8 safety, no unwrap/expect
- Form controls (Switch, RadioButton, Checkbox) grouped in one file as they're closely related
- Animation widgets (ProgressBar, LoadingIndicator) require `tick()` method for frame updates
- Current test count: 1031 tests (from Phase 4.2 completion)
- Expected after Phase 4.3: ~1127 tests

---

## Critical Files for Implementation

- `crates/fae-core/src/widget/tabs.rs` — Tabbed content switcher with keyboard navigation
- `crates/fae-core/src/widget/progress_bar.rs` — Determinate and indeterminate progress indicators
- `crates/fae-core/src/widget/loading_indicator.rs` — Animated loading spinners
- `crates/fae-core/src/widget/form_controls.rs` — Switch, RadioButton, Checkbox widgets
- `crates/fae-core/src/widget/sparkline.rs` — Inline mini-chart for data visualization
