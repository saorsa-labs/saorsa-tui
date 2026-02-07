//! TCSS property name and declaration types.

use crate::tcss::value::CssValue;

/// Supported TCSS property names.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyName {
    // --- Colors ---
    /// Foreground text color.
    Color,
    /// Background color.
    Background,
    /// Border color.
    BorderColor,

    // --- Text decoration ---
    /// Text style (bold, italic, dim, underline, strikethrough, reverse).
    TextStyle,

    // --- Dimensions ---
    /// Widget width.
    Width,
    /// Widget height.
    Height,
    /// Minimum width.
    MinWidth,
    /// Maximum width.
    MaxWidth,
    /// Minimum height.
    MinHeight,
    /// Maximum height.
    MaxHeight,

    // --- Box model: margin ---
    /// Margin on all sides.
    Margin,
    /// Top margin.
    MarginTop,
    /// Right margin.
    MarginRight,
    /// Bottom margin.
    MarginBottom,
    /// Left margin.
    MarginLeft,

    // --- Box model: padding ---
    /// Padding on all sides.
    Padding,
    /// Top padding.
    PaddingTop,
    /// Right padding.
    PaddingRight,
    /// Bottom padding.
    PaddingBottom,
    /// Left padding.
    PaddingLeft,

    // --- Box model: border ---
    /// Border style on all sides.
    Border,
    /// Top border style.
    BorderTop,
    /// Right border style.
    BorderRight,
    /// Bottom border style.
    BorderBottom,
    /// Left border style.
    BorderLeft,

    // --- Layout ---
    /// Display mode (flex, block, none).
    Display,
    /// Flex direction (row, column, row-reverse, column-reverse).
    FlexDirection,
    /// Flex wrap (nowrap, wrap, wrap-reverse).
    FlexWrap,
    /// Justify content (flex-start, flex-end, center, space-between, space-around).
    JustifyContent,
    /// Align items (flex-start, flex-end, center, stretch, baseline).
    AlignItems,
    /// Align self (auto, flex-start, flex-end, center, stretch, baseline).
    AlignSelf,
    /// Flex grow factor.
    FlexGrow,
    /// Flex shrink factor.
    FlexShrink,
    /// Flex basis.
    FlexBasis,
    /// Gap between flex/grid items.
    Gap,

    // --- Grid ---
    /// Grid template columns.
    GridTemplateColumns,
    /// Grid template rows.
    GridTemplateRows,
    /// Grid column placement.
    GridColumn,
    /// Grid row placement.
    GridRow,

    // --- Positioning ---
    /// Dock position (top, bottom, left, right).
    Dock,
    /// Overflow behavior.
    Overflow,
    /// Horizontal overflow behavior.
    OverflowX,
    /// Vertical overflow behavior.
    OverflowY,

    // --- Visibility ---
    /// Visibility (visible, hidden).
    Visibility,
    /// Opacity (0.0 to 1.0, maps to dim).
    Opacity,

    // --- Content alignment ---
    /// Text alignment (left, center, right).
    TextAlign,
    /// Content alignment within container.
    ContentAlign,
}

impl PropertyName {
    /// Parse a CSS property name string into a `PropertyName`.
    ///
    /// Returns `None` if the property name is not recognized.
    /// Matching is case-insensitive.
    pub fn from_css(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "color" => Some(Self::Color),
            "background" | "background-color" => Some(Self::Background),
            "border-color" => Some(Self::BorderColor),
            "text-style" | "text-decoration" => Some(Self::TextStyle),
            "width" => Some(Self::Width),
            "height" => Some(Self::Height),
            "min-width" => Some(Self::MinWidth),
            "max-width" => Some(Self::MaxWidth),
            "min-height" => Some(Self::MinHeight),
            "max-height" => Some(Self::MaxHeight),
            "margin" => Some(Self::Margin),
            "margin-top" => Some(Self::MarginTop),
            "margin-right" => Some(Self::MarginRight),
            "margin-bottom" => Some(Self::MarginBottom),
            "margin-left" => Some(Self::MarginLeft),
            "padding" => Some(Self::Padding),
            "padding-top" => Some(Self::PaddingTop),
            "padding-right" => Some(Self::PaddingRight),
            "padding-bottom" => Some(Self::PaddingBottom),
            "padding-left" => Some(Self::PaddingLeft),
            "border" => Some(Self::Border),
            "border-top" => Some(Self::BorderTop),
            "border-right" => Some(Self::BorderRight),
            "border-bottom" => Some(Self::BorderBottom),
            "border-left" => Some(Self::BorderLeft),
            "display" => Some(Self::Display),
            "flex-direction" => Some(Self::FlexDirection),
            "flex-wrap" => Some(Self::FlexWrap),
            "justify-content" => Some(Self::JustifyContent),
            "align-items" => Some(Self::AlignItems),
            "align-self" => Some(Self::AlignSelf),
            "flex-grow" => Some(Self::FlexGrow),
            "flex-shrink" => Some(Self::FlexShrink),
            "flex-basis" => Some(Self::FlexBasis),
            "gap" => Some(Self::Gap),
            "grid-template-columns" => Some(Self::GridTemplateColumns),
            "grid-template-rows" => Some(Self::GridTemplateRows),
            "grid-column" => Some(Self::GridColumn),
            "grid-row" => Some(Self::GridRow),
            "dock" => Some(Self::Dock),
            "overflow" => Some(Self::Overflow),
            "overflow-x" => Some(Self::OverflowX),
            "overflow-y" => Some(Self::OverflowY),
            "visibility" => Some(Self::Visibility),
            "opacity" => Some(Self::Opacity),
            "text-align" => Some(Self::TextAlign),
            "content-align" => Some(Self::ContentAlign),
            _ => None,
        }
    }

    /// Return the canonical CSS property name.
    pub fn css_name(&self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Background => "background",
            Self::BorderColor => "border-color",
            Self::TextStyle => "text-style",
            Self::Width => "width",
            Self::Height => "height",
            Self::MinWidth => "min-width",
            Self::MaxWidth => "max-width",
            Self::MinHeight => "min-height",
            Self::MaxHeight => "max-height",
            Self::Margin => "margin",
            Self::MarginTop => "margin-top",
            Self::MarginRight => "margin-right",
            Self::MarginBottom => "margin-bottom",
            Self::MarginLeft => "margin-left",
            Self::Padding => "padding",
            Self::PaddingTop => "padding-top",
            Self::PaddingRight => "padding-right",
            Self::PaddingBottom => "padding-bottom",
            Self::PaddingLeft => "padding-left",
            Self::Border => "border",
            Self::BorderTop => "border-top",
            Self::BorderRight => "border-right",
            Self::BorderBottom => "border-bottom",
            Self::BorderLeft => "border-left",
            Self::Display => "display",
            Self::FlexDirection => "flex-direction",
            Self::FlexWrap => "flex-wrap",
            Self::JustifyContent => "justify-content",
            Self::AlignItems => "align-items",
            Self::AlignSelf => "align-self",
            Self::FlexGrow => "flex-grow",
            Self::FlexShrink => "flex-shrink",
            Self::FlexBasis => "flex-basis",
            Self::Gap => "gap",
            Self::GridTemplateColumns => "grid-template-columns",
            Self::GridTemplateRows => "grid-template-rows",
            Self::GridColumn => "grid-column",
            Self::GridRow => "grid-row",
            Self::Dock => "dock",
            Self::Overflow => "overflow",
            Self::OverflowX => "overflow-x",
            Self::OverflowY => "overflow-y",
            Self::Visibility => "visibility",
            Self::Opacity => "opacity",
            Self::TextAlign => "text-align",
            Self::ContentAlign => "content-align",
        }
    }
}

/// A parsed CSS declaration (property: value).
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    /// The property name.
    pub property: PropertyName,
    /// The property value.
    pub value: CssValue,
    /// Whether `!important` was specified.
    pub important: bool,
}

impl Declaration {
    /// Create a new declaration.
    pub fn new(property: PropertyName, value: CssValue) -> Self {
        Self {
            property,
            value,
            important: false,
        }
    }

    /// Create a new declaration with `!important`.
    pub fn important(property: PropertyName, value: CssValue) -> Self {
        Self {
            property,
            value,
            important: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;
    use crate::tcss::value::Length;
    use crate::Color;

    #[test]
    fn from_css_color_properties() {
        assert_eq!(PropertyName::from_css("color"), Some(PropertyName::Color));
        assert_eq!(
            PropertyName::from_css("background"),
            Some(PropertyName::Background)
        );
        assert_eq!(
            PropertyName::from_css("background-color"),
            Some(PropertyName::Background)
        );
        assert_eq!(
            PropertyName::from_css("border-color"),
            Some(PropertyName::BorderColor)
        );
    }

    #[test]
    fn from_css_dimension_properties() {
        assert_eq!(PropertyName::from_css("width"), Some(PropertyName::Width));
        assert_eq!(PropertyName::from_css("height"), Some(PropertyName::Height));
        assert_eq!(
            PropertyName::from_css("min-width"),
            Some(PropertyName::MinWidth)
        );
        assert_eq!(
            PropertyName::from_css("max-height"),
            Some(PropertyName::MaxHeight)
        );
    }

    #[test]
    fn from_css_layout_properties() {
        assert_eq!(
            PropertyName::from_css("display"),
            Some(PropertyName::Display)
        );
        assert_eq!(
            PropertyName::from_css("flex-direction"),
            Some(PropertyName::FlexDirection)
        );
        assert_eq!(
            PropertyName::from_css("justify-content"),
            Some(PropertyName::JustifyContent)
        );
        assert_eq!(
            PropertyName::from_css("align-items"),
            Some(PropertyName::AlignItems)
        );
    }

    #[test]
    fn from_css_box_model() {
        assert_eq!(PropertyName::from_css("margin"), Some(PropertyName::Margin));
        assert_eq!(
            PropertyName::from_css("padding-top"),
            Some(PropertyName::PaddingTop)
        );
        assert_eq!(PropertyName::from_css("border"), Some(PropertyName::Border));
    }

    #[test]
    fn from_css_grid_properties() {
        assert_eq!(
            PropertyName::from_css("grid-template-columns"),
            Some(PropertyName::GridTemplateColumns)
        );
        assert_eq!(
            PropertyName::from_css("grid-row"),
            Some(PropertyName::GridRow)
        );
    }

    #[test]
    fn from_css_case_insensitive() {
        assert_eq!(PropertyName::from_css("COLOR"), Some(PropertyName::Color));
        assert_eq!(
            PropertyName::from_css("Flex-Direction"),
            Some(PropertyName::FlexDirection)
        );
    }

    #[test]
    fn from_css_unknown_returns_none() {
        assert_eq!(PropertyName::from_css("unknown-prop"), None);
        assert_eq!(PropertyName::from_css(""), None);
        assert_eq!(PropertyName::from_css("not-a-property"), None);
    }

    #[test]
    fn css_name_round_trip() {
        let properties = [
            PropertyName::Color,
            PropertyName::Background,
            PropertyName::Width,
            PropertyName::FlexDirection,
            PropertyName::GridTemplateColumns,
            PropertyName::Dock,
            PropertyName::Opacity,
        ];
        for prop in &properties {
            let name = prop.css_name();
            let parsed = PropertyName::from_css(name);
            assert_eq!(parsed.as_ref(), Some(prop), "round-trip failed for {name}");
        }
    }

    #[test]
    fn declaration_new() {
        let decl = Declaration::new(
            PropertyName::Color,
            CssValue::Color(Color::Named(NamedColor::Red)),
        );
        assert_eq!(decl.property, PropertyName::Color);
        assert!(!decl.important);
    }

    #[test]
    fn declaration_important() {
        let decl = Declaration::important(
            PropertyName::Width,
            CssValue::Length(Length::Cells(10)),
        );
        assert_eq!(decl.property, PropertyName::Width);
        assert!(decl.important);
    }
}
