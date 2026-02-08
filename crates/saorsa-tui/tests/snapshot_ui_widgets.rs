//! Snapshot tests for UI control widgets.

use saorsa_tui::buffer::ScreenBuffer;
use saorsa_tui::geometry::{Rect, Size};
use saorsa_tui::segment::Segment;
use saorsa_tui::widget::{
    Checkbox, Collapsible, IndicatorStyle, LoadingIndicator, OptionList, ProgressBar, RadioButton,
    Sparkline, Switch, Tab, Tabs, Widget,
};

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

// --- Tabs Snapshots ---

#[test]
fn snapshot_tabs_two_tabs_first_selected() {
    let tabs = Tabs::new(vec![
        Tab::new("Overview").with_content(vec![vec![Segment::new("Overview content here")]]),
        Tab::new("Details").with_content(vec![vec![Segment::new("Details content here")]]),
    ]);
    let rendered = render_widget_to_text(&tabs, 50, 4);
    insta::assert_snapshot!("tabs_two_tabs_first_selected", rendered);
}

#[test]
fn snapshot_tabs_multiple_tabs() {
    let tabs = Tabs::new(vec![
        Tab::new("Home").with_content(vec![vec![Segment::new("Home page")]]),
        Tab::new("Settings").with_content(vec![vec![Segment::new("Settings page")]]),
        Tab::new("Help").with_content(vec![vec![Segment::new("Help page")]]),
    ]);
    let rendered = render_widget_to_text(&tabs, 60, 5);
    insta::assert_snapshot!("tabs_multiple_tabs", rendered);
}

// --- ProgressBar Snapshots ---

#[test]
fn snapshot_progress_bar_0_percent() {
    let bar = ProgressBar::new(0.0).with_show_percentage(true);
    let rendered = render_widget_to_text(&bar, 30, 1);
    insta::assert_snapshot!("progress_bar_0_percent", rendered);
}

#[test]
fn snapshot_progress_bar_50_percent() {
    let bar = ProgressBar::new(0.5).with_show_percentage(true);
    let rendered = render_widget_to_text(&bar, 30, 1);
    insta::assert_snapshot!("progress_bar_50_percent", rendered);
}

#[test]
fn snapshot_progress_bar_100_percent() {
    let bar = ProgressBar::new(1.0).with_show_percentage(true);
    let rendered = render_widget_to_text(&bar, 30, 1);
    insta::assert_snapshot!("progress_bar_100_percent", rendered);
}

#[test]
fn snapshot_progress_bar_indeterminate() {
    let bar = ProgressBar::indeterminate();
    let rendered = render_widget_to_text(&bar, 30, 1);
    insta::assert_snapshot!("progress_bar_indeterminate", rendered);
}

// --- LoadingIndicator Snapshots ---

#[test]
fn snapshot_loading_indicator_spinner() {
    let loader = LoadingIndicator::new()
        .with_style(IndicatorStyle::Spinner)
        .with_message("Loading...");
    let rendered = render_widget_to_text(&loader, 30, 1);
    insta::assert_snapshot!("loading_indicator_spinner", rendered);
}

#[test]
fn snapshot_loading_indicator_dots() {
    let loader = LoadingIndicator::new()
        .with_style(IndicatorStyle::Dots)
        .with_message("Processing...");
    let rendered = render_widget_to_text(&loader, 30, 1);
    insta::assert_snapshot!("loading_indicator_dots", rendered);
}

#[test]
fn snapshot_loading_indicator_line() {
    let loader = LoadingIndicator::new()
        .with_style(IndicatorStyle::Line)
        .with_message("Please wait");
    let rendered = render_widget_to_text(&loader, 30, 1);
    insta::assert_snapshot!("loading_indicator_line", rendered);
}

#[test]
fn snapshot_loading_indicator_circle() {
    let loader = LoadingIndicator::new()
        .with_style(IndicatorStyle::Circle)
        .with_message("Working...");
    let rendered = render_widget_to_text(&loader, 30, 1);
    insta::assert_snapshot!("loading_indicator_circle", rendered);
}

// --- Collapsible Snapshots ---

#[test]
fn snapshot_collapsible_collapsed() {
    let collapsible = Collapsible::new("Section Title").with_content(vec![
        vec![Segment::new("Line 1 content")],
        vec![Segment::new("Line 2 content")],
    ]);
    let rendered = render_widget_to_text(&collapsible, 40, 5);
    insta::assert_snapshot!("collapsible_collapsed", rendered);
}

#[test]
fn snapshot_collapsible_expanded() {
    let collapsible = Collapsible::new("Section Title")
        .with_content(vec![
            vec![Segment::new("Line 1 content")],
            vec![Segment::new("Line 2 content")],
            vec![Segment::new("Line 3 content")],
        ])
        .with_expanded(true);
    let rendered = render_widget_to_text(&collapsible, 40, 6);
    insta::assert_snapshot!("collapsible_expanded", rendered);
}

// --- Form Controls Snapshots ---

#[test]
fn snapshot_switch_off() {
    let switch = Switch::new("Dark Mode");
    let rendered = render_widget_to_text(&switch, 30, 1);
    insta::assert_snapshot!("switch_off", rendered);
}

#[test]
fn snapshot_switch_on() {
    let switch = Switch::new("Dark Mode").with_state(true);
    let rendered = render_widget_to_text(&switch, 30, 1);
    insta::assert_snapshot!("switch_on", rendered);
}

#[test]
fn snapshot_radio_button_unselected() {
    let radio = RadioButton::new("Option A");
    let rendered = render_widget_to_text(&radio, 30, 1);
    insta::assert_snapshot!("radio_button_unselected", rendered);
}

#[test]
fn snapshot_radio_button_selected() {
    let radio = RadioButton::new("Option A").with_selected(true);
    let rendered = render_widget_to_text(&radio, 30, 1);
    insta::assert_snapshot!("radio_button_selected", rendered);
}

#[test]
fn snapshot_checkbox_unchecked() {
    let checkbox = Checkbox::new("I agree to the terms");
    let rendered = render_widget_to_text(&checkbox, 35, 1);
    insta::assert_snapshot!("checkbox_unchecked", rendered);
}

#[test]
fn snapshot_checkbox_checked() {
    let checkbox = Checkbox::new("I agree to the terms").with_checked(true);
    let rendered = render_widget_to_text(&checkbox, 35, 1);
    insta::assert_snapshot!("checkbox_checked", rendered);
}

// --- Sparkline Snapshots ---

#[test]
fn snapshot_sparkline_increasing() {
    let sparkline = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let rendered = render_widget_to_text(&sparkline, 20, 1);
    insta::assert_snapshot!("sparkline_increasing", rendered);
}

#[test]
fn snapshot_sparkline_wave() {
    let sparkline = Sparkline::new(vec![1.0, 3.0, 2.0, 4.0, 2.0, 3.0, 1.0, 2.0]);
    let rendered = render_widget_to_text(&sparkline, 20, 1);
    insta::assert_snapshot!("sparkline_wave", rendered);
}

#[test]
fn snapshot_sparkline_flat() {
    let sparkline = Sparkline::new(vec![5.0, 5.0, 5.0, 5.0, 5.0]);
    let rendered = render_widget_to_text(&sparkline, 20, 1);
    insta::assert_snapshot!("sparkline_flat", rendered);
}

// --- OptionList Snapshots ---

#[test]
fn snapshot_option_list_basic() {
    let option_list = OptionList::new(vec![
        "Option 1".to_string(),
        "Option 2".to_string(),
        "Option 3".to_string(),
        "Option 4".to_string(),
    ]);
    let rendered = render_widget_to_text(&option_list, 30, 6);
    insta::assert_snapshot!("option_list_basic", rendered);
}

#[test]
fn snapshot_option_list_with_prefix() {
    let option_list = OptionList::new(vec![
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
    ])
    .with_prefix("> ");
    let rendered = render_widget_to_text(&option_list, 30, 5);
    insta::assert_snapshot!("option_list_with_prefix", rendered);
}
