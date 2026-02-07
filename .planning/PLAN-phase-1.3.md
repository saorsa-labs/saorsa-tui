# Phase 1.3: Basic Layout & Widgets

## Overview
Implement the event loop, basic layout system (vertical/horizontal stacking,
dock positioning), and foundational widgets (Static, Label, Container,
ScrollView, Input). Add focus management.

## Tasks

### Task 1: Event System
- `crates/saorsa-core/src/event.rs`
- `Event` enum: Key(KeyEvent), Mouse(MouseEvent), Resize(u16, u16), Paste(String)
- `KeyEvent` struct: code(KeyCode), modifiers(Modifiers)
- `KeyCode` enum: Char(char), Enter, Tab, Backspace, Escape, Up/Down/Left/Right, etc.
- `Modifiers` bitflags: SHIFT, CTRL, ALT, SUPER
- `MouseEvent`: kind(Press/Release/Move/Scroll), x, y, modifiers
- Crossterm event conversion: From<crossterm::event::Event>
- Tests: event creation, modifier checks, crossterm conversion

### Task 2: Event Loop
- `crates/saorsa-core/src/event_loop.rs`
- `EventLoop` that polls crossterm events and dispatches them
- Integration with RenderContext for frame-based rendering
- Resize event triggers buffer resize
- Quit handling (Ctrl+C, Ctrl+Q)
- Tick/frame rate control
- Tests: event dispatch (using TestBackend + synthetic events)

### Task 3: Widget Trait
- `crates/saorsa-core/src/widget/mod.rs`
- `Widget` trait: render(&self, area: Rect, buf: &mut ScreenBuffer)
- `SizedWidget` trait: min_size(), max_size(), preferred_size()
- `InteractiveWidget` trait: handle_event(&mut self, event: &Event) -> EventResult
- `EventResult` enum: Consumed, Ignored
- `WidgetId` type (u64 with generation counter)
- Tests: widget trait impl with mock widget

### Task 4: Layout System
- `crates/saorsa-core/src/layout.rs`
- `Direction` enum: Vertical, Horizontal
- `Constraint` enum: Fixed(u16), Min(u16), Max(u16), Percentage(u8), Fill
- `Layout::split(area: Rect, direction: Direction, constraints: &[Constraint]) -> Vec<Rect>`
- Constraint solving: allocate fixed first, then percentages, then fill remaining
- `Dock` enum: Top, Bottom, Left, Right
- `Layout::dock(area: Rect, dock: Dock, size: u16) -> (Rect, Rect)` — returns (docked, remaining)
- Tests: vertical split, horizontal split, fixed+fill, percentages, dock positions

### Task 5: Label Widget
- `crates/saorsa-core/src/widget/label.rs`
- Display a single line of styled text
- Alignment: Left, Center, Right
- Truncation with ellipsis when text exceeds width
- Implements Widget trait
- Tests: rendering, alignment, truncation

### Task 6: Static Widget
- `crates/saorsa-core/src/widget/static_widget.rs`
- Display pre-rendered content (Vec<Segment>) without interaction
- Multi-line support
- Implements Widget trait
- Tests: rendering single and multi-line content

### Task 7: Container Widget
- `crates/saorsa-core/src/widget/container.rs`
- Box with optional border, title, padding
- `BorderStyle` enum: None, Single, Double, Rounded, Heavy
- Border characters: ┌┐└┘─│ (single), ╔╗╚╝═║ (double), ╭╮╰╯─│ (rounded)
- Renders border, then delegates inner area to child
- Tests: border rendering, padding, title

### Task 8: Focus Management
- `crates/saorsa-core/src/focus.rs`
- `FocusManager` tracks which widget has focus
- Tab/Shift-Tab navigation order
- Focus ring (wraps around)
- `FocusState`: Focused, Unfocused
- Tests: focus cycling, tab navigation

### Task 9: Wire Up & Integration
- Add all new modules to lib.rs
- Re-export key types
- Integration test: layout + container + label rendering to TestBackend
- All existing tests still pass
- Zero clippy warnings

## Dependencies
- Phase 1.2 complete (ScreenBuffer, Renderer, RenderContext) ✅

## Acceptance Criteria
- Event system converts crossterm events to our types
- Layout splits areas correctly (vertical, horizontal, dock)
- Label renders with alignment and truncation
- Static renders pre-made segments
- Container draws borders and delegates to child
- Focus manager cycles through widgets
- All tests pass, zero clippy warnings
