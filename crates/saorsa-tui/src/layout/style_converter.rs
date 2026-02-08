//! Converts [`ComputedStyle`] to [`taffy::Style`].
//!
//! Maps TCSS property values to Taffy layout types for Flexbox and Grid
//! layout computation.

use taffy::Overflow;
use taffy::prelude::*;

use crate::tcss::cascade::ComputedStyle;
use crate::tcss::property::PropertyName;
use crate::tcss::value::{CssValue, Length};

/// Convert a [`ComputedStyle`] into a [`taffy::Style`].
///
/// Maps TCSS property names to the corresponding Taffy style fields.
/// Properties not present in the computed style fall back to Taffy defaults.
pub fn computed_to_taffy(computed: &ComputedStyle) -> Style {
    let mut style = Style::default();

    if let Some(v) = computed.get(&PropertyName::Display) {
        style.display = to_display(v);
    }
    if let Some(v) = computed.get(&PropertyName::FlexDirection) {
        style.flex_direction = to_flex_direction(v);
    }
    if let Some(v) = computed.get(&PropertyName::FlexWrap) {
        style.flex_wrap = to_flex_wrap(v);
    }
    if let Some(v) = computed.get(&PropertyName::JustifyContent) {
        style.justify_content = to_justify_content(v);
    }
    if let Some(v) = computed.get(&PropertyName::AlignItems) {
        style.align_items = to_align_items(v);
    }
    if let Some(v) = computed.get(&PropertyName::AlignSelf) {
        style.align_self = to_align_self(v);
    }

    // Flex factors
    if let Some(v) = computed.get(&PropertyName::FlexGrow) {
        style.flex_grow = to_f32(v);
    }
    if let Some(v) = computed.get(&PropertyName::FlexShrink) {
        style.flex_shrink = to_f32(v);
    }
    if let Some(v) = computed.get(&PropertyName::FlexBasis) {
        style.flex_basis = to_dimension(v);
    }

    // Gap
    if let Some(v) = computed.get(&PropertyName::Gap) {
        let lp = to_length_percentage(v);
        style.gap = taffy::Size {
            width: lp,
            height: lp,
        };
    }

    // Size
    if let Some(v) = computed.get(&PropertyName::Width) {
        style.size.width = to_dimension(v);
    }
    if let Some(v) = computed.get(&PropertyName::Height) {
        style.size.height = to_dimension(v);
    }

    // Min/Max size
    if let Some(v) = computed.get(&PropertyName::MinWidth) {
        style.min_size.width = to_dimension(v);
    }
    if let Some(v) = computed.get(&PropertyName::MinHeight) {
        style.min_size.height = to_dimension(v);
    }
    if let Some(v) = computed.get(&PropertyName::MaxWidth) {
        style.max_size.width = to_dimension(v);
    }
    if let Some(v) = computed.get(&PropertyName::MaxHeight) {
        style.max_size.height = to_dimension(v);
    }

    // Margin
    apply_margin(computed, &mut style);

    // Padding
    apply_padding(computed, &mut style);

    // Border
    apply_border(computed, &mut style);

    // Overflow
    apply_overflow(computed, &mut style);

    // Grid templates
    if let Some(v) = computed.get(&PropertyName::GridTemplateColumns) {
        style.grid_template_columns = to_grid_tracks(v);
    }
    if let Some(v) = computed.get(&PropertyName::GridTemplateRows) {
        style.grid_template_rows = to_grid_tracks(v);
    }

    // Grid placement
    if let Some(v) = computed.get(&PropertyName::GridColumn) {
        style.grid_column = to_grid_placement(v);
    }
    if let Some(v) = computed.get(&PropertyName::GridRow) {
        style.grid_row = to_grid_placement(v);
    }

    style
}

/// Convert a CSS value to a Taffy [`Dimension`].
pub fn to_dimension(value: &CssValue) -> Dimension {
    match value {
        CssValue::Length(Length::Cells(n)) => Dimension::Length(f32::from(*n)),
        CssValue::Length(Length::Percent(p)) => Dimension::Percent(*p / 100.0),
        CssValue::Length(Length::Auto) => Dimension::Auto,
        CssValue::Integer(n) => Dimension::Length(*n as f32),
        CssValue::Float(f) => Dimension::Length(*f),
        CssValue::Keyword(k) if k.eq_ignore_ascii_case("auto") => Dimension::Auto,
        _ => Dimension::Auto,
    }
}

/// Convert a CSS value to a Taffy [`LengthPercentage`].
pub fn to_length_percentage(value: &CssValue) -> LengthPercentage {
    match value {
        CssValue::Length(Length::Cells(n)) => LengthPercentage::Length(f32::from(*n)),
        CssValue::Length(Length::Percent(p)) => LengthPercentage::Percent(*p / 100.0),
        CssValue::Integer(n) => LengthPercentage::Length(*n as f32),
        CssValue::Float(f) => LengthPercentage::Length(*f),
        _ => LengthPercentage::Length(0.0),
    }
}

/// Convert a CSS value to a Taffy [`LengthPercentageAuto`].
pub fn to_length_percentage_auto(value: &CssValue) -> LengthPercentageAuto {
    match value {
        CssValue::Length(Length::Cells(n)) => LengthPercentageAuto::Length(f32::from(*n)),
        CssValue::Length(Length::Percent(p)) => LengthPercentageAuto::Percent(*p / 100.0),
        CssValue::Length(Length::Auto) => LengthPercentageAuto::Auto,
        CssValue::Integer(n) => LengthPercentageAuto::Length(*n as f32),
        CssValue::Float(f) => LengthPercentageAuto::Length(*f),
        CssValue::Keyword(k) if k.eq_ignore_ascii_case("auto") => LengthPercentageAuto::Auto,
        _ => LengthPercentageAuto::Auto,
    }
}

/// Convert a CSS value to a Taffy [`Display`].
pub fn to_display(value: &CssValue) -> Display {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "flex" => Display::Flex,
            "grid" => Display::Grid,
            "block" => Display::Block,
            "none" => Display::None,
            _ => Display::Flex,
        },
        _ => Display::Flex,
    }
}

/// Convert a CSS value to a Taffy [`FlexDirection`].
pub fn to_flex_direction(value: &CssValue) -> FlexDirection {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "row" => FlexDirection::Row,
            "column" => FlexDirection::Column,
            "row-reverse" => FlexDirection::RowReverse,
            "column-reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        },
        _ => FlexDirection::Row,
    }
}

/// Convert a CSS value to a Taffy [`FlexWrap`].
pub fn to_flex_wrap(value: &CssValue) -> FlexWrap {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "nowrap" => FlexWrap::NoWrap,
            "wrap" => FlexWrap::Wrap,
            "wrap-reverse" => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        },
        _ => FlexWrap::NoWrap,
    }
}

/// Convert a CSS value to a Taffy [`JustifyContent`].
pub fn to_justify_content(value: &CssValue) -> Option<JustifyContent> {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "flex-start" | "start" => Some(JustifyContent::Start),
            "flex-end" | "end" => Some(JustifyContent::End),
            "center" => Some(JustifyContent::Center),
            "space-between" => Some(JustifyContent::SpaceBetween),
            "space-around" => Some(JustifyContent::SpaceAround),
            "space-evenly" => Some(JustifyContent::SpaceEvenly),
            _ => None,
        },
        _ => None,
    }
}

/// Convert a CSS value to a Taffy [`AlignItems`].
pub fn to_align_items(value: &CssValue) -> Option<AlignItems> {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "flex-start" | "start" => Some(AlignItems::Start),
            "flex-end" | "end" => Some(AlignItems::End),
            "center" => Some(AlignItems::Center),
            "stretch" => Some(AlignItems::Stretch),
            "baseline" => Some(AlignItems::Baseline),
            _ => None,
        },
        _ => None,
    }
}

/// Convert a CSS value to a Taffy [`AlignSelf`].
pub fn to_align_self(value: &CssValue) -> Option<AlignSelf> {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "flex-start" | "start" => Some(AlignSelf::Start),
            "flex-end" | "end" => Some(AlignSelf::End),
            "center" => Some(AlignSelf::Center),
            "stretch" => Some(AlignSelf::Stretch),
            "baseline" => Some(AlignSelf::Baseline),
            _ => None,
        },
        _ => None,
    }
}

/// Convert a CSS value to a Taffy [`Overflow`].
pub fn to_overflow(value: &CssValue) -> Overflow {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "visible" => Overflow::Visible,
            "hidden" => Overflow::Hidden,
            "scroll" => Overflow::Scroll,
            "auto" => Overflow::Scroll,
            "clip" => Overflow::Clip,
            _ => Overflow::Visible,
        },
        _ => Overflow::Visible,
    }
}

/// Convert a CSS value to a list of Taffy [`TrackSizingFunction`] for grid.
pub fn to_grid_tracks(value: &CssValue) -> Vec<TrackSizingFunction> {
    match value {
        CssValue::List(items) => items.iter().map(single_track).collect(),
        other => vec![single_track(other)],
    }
}

/// Convert a CSS value to a Taffy [`GridPlacement`].
pub fn to_grid_placement(value: &CssValue) -> Line<GridPlacement> {
    match value {
        CssValue::Integer(n) => Line {
            start: GridPlacement::from_line_index(i16::try_from(*n).unwrap_or(1)),
            end: GridPlacement::Auto,
        },
        CssValue::Keyword(k) => {
            let lower = k.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("span ")
                && let Ok(n) = rest.trim().parse::<u16>()
            {
                return Line {
                    start: GridPlacement::from_span(n),
                    end: GridPlacement::Auto,
                };
            }
            // Handle "start / end" format
            if let Some((start_str, end_str)) = lower.split_once('/') {
                let start = start_str
                    .trim()
                    .parse::<i16>()
                    .map(GridPlacement::from_line_index)
                    .unwrap_or(GridPlacement::Auto);
                let end = end_str
                    .trim()
                    .parse::<i16>()
                    .map(GridPlacement::from_line_index)
                    .unwrap_or(GridPlacement::Auto);
                return Line { start, end };
            }
            Line {
                start: GridPlacement::Auto,
                end: GridPlacement::Auto,
            }
        }
        _ => Line {
            start: GridPlacement::Auto,
            end: GridPlacement::Auto,
        },
    }
}

// --- Private helpers ---

/// Extract an f32 from a CSS value.
fn to_f32(value: &CssValue) -> f32 {
    match value {
        CssValue::Integer(n) => *n as f32,
        CssValue::Float(f) => *f,
        _ => 0.0,
    }
}

/// Convert a single CSS value to a [`TrackSizingFunction`].
fn single_track(value: &CssValue) -> TrackSizingFunction {
    match value {
        CssValue::Fr(f) => fr(*f),
        CssValue::Length(Length::Cells(n)) => length(f32::from(*n)),
        CssValue::Length(Length::Percent(p)) => percent(*p / 100.0),
        CssValue::Length(Length::Auto) => auto(),
        CssValue::Integer(n) => length(*n as f32),
        CssValue::Float(f) => length(*f),
        CssValue::Keyword(k) if k.eq_ignore_ascii_case("auto") => auto(),
        _ => auto(),
    }
}

/// Apply margin properties from computed style.
fn apply_margin(computed: &ComputedStyle, style: &mut Style) {
    if let Some(v) = computed.get(&PropertyName::Margin) {
        let lpa = to_length_percentage_auto(v);
        style.margin = taffy::Rect {
            left: lpa,
            right: lpa,
            top: lpa,
            bottom: lpa,
        };
    }
    if let Some(v) = computed.get(&PropertyName::MarginTop) {
        style.margin.top = to_length_percentage_auto(v);
    }
    if let Some(v) = computed.get(&PropertyName::MarginRight) {
        style.margin.right = to_length_percentage_auto(v);
    }
    if let Some(v) = computed.get(&PropertyName::MarginBottom) {
        style.margin.bottom = to_length_percentage_auto(v);
    }
    if let Some(v) = computed.get(&PropertyName::MarginLeft) {
        style.margin.left = to_length_percentage_auto(v);
    }
}

/// Apply padding properties from computed style.
fn apply_padding(computed: &ComputedStyle, style: &mut Style) {
    if let Some(v) = computed.get(&PropertyName::Padding) {
        let lp = to_length_percentage(v);
        style.padding = taffy::Rect {
            left: lp,
            right: lp,
            top: lp,
            bottom: lp,
        };
    }
    if let Some(v) = computed.get(&PropertyName::PaddingTop) {
        style.padding.top = to_length_percentage(v);
    }
    if let Some(v) = computed.get(&PropertyName::PaddingRight) {
        style.padding.right = to_length_percentage(v);
    }
    if let Some(v) = computed.get(&PropertyName::PaddingBottom) {
        style.padding.bottom = to_length_percentage(v);
    }
    if let Some(v) = computed.get(&PropertyName::PaddingLeft) {
        style.padding.left = to_length_percentage(v);
    }
}

/// Apply border properties from computed style.
///
/// In terminal mode, borders are always 1 cell wide when present.
fn apply_border(computed: &ComputedStyle, style: &mut Style) {
    if let Some(_v) = computed.get(&PropertyName::Border) {
        style.border = taffy::Rect {
            left: LengthPercentage::Length(1.0),
            right: LengthPercentage::Length(1.0),
            top: LengthPercentage::Length(1.0),
            bottom: LengthPercentage::Length(1.0),
        };
    }
    if computed.get(&PropertyName::BorderTop).is_some() {
        style.border.top = LengthPercentage::Length(1.0);
    }
    if computed.get(&PropertyName::BorderRight).is_some() {
        style.border.right = LengthPercentage::Length(1.0);
    }
    if computed.get(&PropertyName::BorderBottom).is_some() {
        style.border.bottom = LengthPercentage::Length(1.0);
    }
    if computed.get(&PropertyName::BorderLeft).is_some() {
        style.border.left = LengthPercentage::Length(1.0);
    }
}

/// Apply overflow properties from computed style.
fn apply_overflow(computed: &ComputedStyle, style: &mut Style) {
    if let Some(v) = computed.get(&PropertyName::Overflow) {
        let o = to_overflow(v);
        style.overflow = taffy::Point { x: o, y: o };
    }
    if let Some(v) = computed.get(&PropertyName::OverflowX) {
        style.overflow.x = to_overflow(v);
    }
    if let Some(v) = computed.get(&PropertyName::OverflowY) {
        style.overflow.y = to_overflow(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcss::cascade::ComputedStyle;
    use crate::tcss::property::PropertyName;
    use crate::tcss::value::{CssValue, Length};

    #[test]
    fn convert_empty_style() {
        let computed = ComputedStyle::new();
        let style = computed_to_taffy(&computed);
        // Default should be Flex display
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn convert_display_flex() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Display, CssValue::Keyword("flex".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn convert_display_grid() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Display, CssValue::Keyword("grid".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.display, Display::Grid);
    }

    #[test]
    fn convert_display_none() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Display, CssValue::Keyword("none".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.display, Display::None);
    }

    #[test]
    fn convert_flex_direction() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyName::FlexDirection,
            CssValue::Keyword("column".into()),
        );
        let style = computed_to_taffy(&computed);
        assert_eq!(style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn convert_flex_wrap() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::FlexWrap, CssValue::Keyword("wrap".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.flex_wrap, FlexWrap::Wrap);
    }

    #[test]
    fn convert_justify_content() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyName::JustifyContent,
            CssValue::Keyword("center".into()),
        );
        let style = computed_to_taffy(&computed);
        assert_eq!(style.justify_content, Some(JustifyContent::Center));
    }

    #[test]
    fn convert_align_items() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyName::AlignItems,
            CssValue::Keyword("stretch".into()),
        );
        let style = computed_to_taffy(&computed);
        assert_eq!(style.align_items, Some(AlignItems::Stretch));
    }

    #[test]
    fn convert_flex_grow_shrink() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::FlexGrow, CssValue::Integer(2));
        computed.set(PropertyName::FlexShrink, CssValue::Float(0.5));
        let style = computed_to_taffy(&computed);
        assert!((style.flex_grow - 2.0).abs() < f32::EPSILON);
        assert!((style.flex_shrink - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn convert_dimensions() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Width, CssValue::Length(Length::Cells(80)));
        computed.set(PropertyName::Height, CssValue::Length(Length::Cells(24)));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.size.width, Dimension::Length(80.0));
        assert_eq!(style.size.height, Dimension::Length(24.0));
    }

    #[test]
    fn convert_min_max_size() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::MinWidth, CssValue::Length(Length::Cells(10)));
        computed.set(
            PropertyName::MaxHeight,
            CssValue::Length(Length::Percent(50.0)),
        );
        let style = computed_to_taffy(&computed);
        assert_eq!(style.min_size.width, Dimension::Length(10.0));
        assert_eq!(style.max_size.height, Dimension::Percent(0.5));
    }

    #[test]
    fn convert_margin_all_sides() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Margin, CssValue::Length(Length::Cells(5)));
        let style = computed_to_taffy(&computed);
        let expected = LengthPercentageAuto::Length(5.0);
        assert_eq!(style.margin.top, expected);
        assert_eq!(style.margin.right, expected);
        assert_eq!(style.margin.bottom, expected);
        assert_eq!(style.margin.left, expected);
    }

    #[test]
    fn convert_margin_individual_overrides() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Margin, CssValue::Length(Length::Cells(5)));
        computed.set(PropertyName::MarginTop, CssValue::Length(Length::Cells(10)));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.margin.top, LengthPercentageAuto::Length(10.0));
        assert_eq!(style.margin.right, LengthPercentageAuto::Length(5.0));
    }

    #[test]
    fn convert_padding_individual() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::PaddingTop, CssValue::Length(Length::Cells(2)));
        computed.set(
            PropertyName::PaddingLeft,
            CssValue::Length(Length::Cells(4)),
        );
        let style = computed_to_taffy(&computed);
        assert_eq!(style.padding.top, LengthPercentage::Length(2.0));
        assert_eq!(style.padding.left, LengthPercentage::Length(4.0));
    }

    #[test]
    fn convert_border_width() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Border, CssValue::Keyword("solid".into()));
        let style = computed_to_taffy(&computed);
        let one = LengthPercentage::Length(1.0);
        assert_eq!(style.border.top, one);
        assert_eq!(style.border.right, one);
        assert_eq!(style.border.bottom, one);
        assert_eq!(style.border.left, one);
    }

    #[test]
    fn convert_overflow_hidden() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Overflow, CssValue::Keyword("hidden".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.overflow.x, Overflow::Hidden);
        assert_eq!(style.overflow.y, Overflow::Hidden);
    }

    #[test]
    fn convert_overflow_xy_separate() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::OverflowX, CssValue::Keyword("scroll".into()));
        computed.set(PropertyName::OverflowY, CssValue::Keyword("hidden".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.overflow.x, Overflow::Scroll);
        assert_eq!(style.overflow.y, Overflow::Hidden);
    }

    #[test]
    fn convert_grid_template_fr() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyName::GridTemplateColumns,
            CssValue::List(vec![CssValue::Fr(1.0), CssValue::Fr(2.0)]),
        );
        let style = computed_to_taffy(&computed);
        assert_eq!(style.grid_template_columns.len(), 2);
    }

    #[test]
    fn convert_grid_placement_span() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::GridColumn, CssValue::Keyword("span 3".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.grid_column.start, GridPlacement::from_span(3));
    }

    #[test]
    fn convert_grid_placement_line() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::GridRow, CssValue::Integer(2));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.grid_row.start, GridPlacement::from_line_index(2));
    }

    #[test]
    fn convert_gap() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::Gap, CssValue::Length(Length::Cells(2)));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.gap.width, LengthPercentage::Length(2.0));
        assert_eq!(style.gap.height, LengthPercentage::Length(2.0));
    }

    #[test]
    fn convert_percentage_dimension() {
        let dim = to_dimension(&CssValue::Length(Length::Percent(50.0)));
        assert_eq!(dim, Dimension::Percent(0.5));
    }

    #[test]
    fn convert_auto_dimension() {
        let dim = to_dimension(&CssValue::Length(Length::Auto));
        assert_eq!(dim, Dimension::Auto);
    }

    #[test]
    fn convert_align_self() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyName::AlignSelf, CssValue::Keyword("center".into()));
        let style = computed_to_taffy(&computed);
        assert_eq!(style.align_self, Some(AlignSelf::Center));
    }

    #[test]
    fn to_display_block() {
        assert_eq!(
            to_display(&CssValue::Keyword("block".into())),
            Display::Block
        );
    }

    #[test]
    fn to_flex_direction_row_reverse() {
        assert_eq!(
            to_flex_direction(&CssValue::Keyword("row-reverse".into())),
            FlexDirection::RowReverse
        );
    }

    #[test]
    fn to_flex_wrap_reverse() {
        assert_eq!(
            to_flex_wrap(&CssValue::Keyword("wrap-reverse".into())),
            FlexWrap::WrapReverse
        );
    }

    #[test]
    fn to_justify_space_evenly() {
        assert_eq!(
            to_justify_content(&CssValue::Keyword("space-evenly".into())),
            Some(JustifyContent::SpaceEvenly)
        );
    }

    #[test]
    fn to_align_items_baseline() {
        assert_eq!(
            to_align_items(&CssValue::Keyword("baseline".into())),
            Some(AlignItems::Baseline)
        );
    }

    #[test]
    fn to_overflow_scroll() {
        assert_eq!(
            to_overflow(&CssValue::Keyword("scroll".into())),
            Overflow::Scroll
        );
    }

    #[test]
    fn to_overflow_auto() {
        assert_eq!(
            to_overflow(&CssValue::Keyword("auto".into())),
            Overflow::Scroll
        );
    }

    #[test]
    fn grid_template_single_value() {
        let tracks = to_grid_tracks(&CssValue::Fr(1.0));
        assert_eq!(tracks.len(), 1);
    }

    #[test]
    fn grid_placement_range() {
        let placement = to_grid_placement(&CssValue::Keyword("1 / 3".into()));
        assert_eq!(placement.start, GridPlacement::from_line_index(1));
        assert_eq!(placement.end, GridPlacement::from_line_index(3));
    }
}
