//! TCSS stylesheet parser.
//!
//! Parses TCSS stylesheets into the AST using the `cssparser` crate.

use cssparser::{Parser, Token};

use crate::color::Color;
use crate::tcss::error::TcssError;
use crate::tcss::property::PropertyName;
use crate::tcss::value::{CssValue, Length};

/// Parse a color value from CSS input.
///
/// Supports:
/// - Named colors: `red`, `blue`, `green`, etc.
/// - Hex colors: `#rgb`, `#rrggbb`
/// - `rgb(r, g, b)` function
pub fn parse_color(input: &mut Parser<'_, '_>) -> Result<Color, TcssError> {
    let token = input
        .next()
        .map_err(|e| TcssError::Parse(format!("{e:?}")))?
        .clone();
    match &token {
        Token::Ident(name) => {
            let name_str = name.to_string();
            Color::from_css_name(&name_str).ok_or_else(|| TcssError::InvalidValue {
                property: "color".into(),
                value: name_str,
            })
        }
        Token::Hash(hash) | Token::IDHash(hash) => {
            let hash_str = hash.to_string();
            Color::from_hex(&hash_str).map_err(|e| TcssError::Parse(e.to_string()))
        }
        Token::Function(name) if name.eq_ignore_ascii_case("rgb") => parse_rgb_block(input),
        other => Err(TcssError::Parse(format!("expected color, got {other:?}"))),
    }
}

/// Parse the inside of an `rgb(r, g, b)` function call.
fn parse_rgb_block(input: &mut Parser<'_, '_>) -> Result<Color, TcssError> {
    let result: Result<(i32, i32, i32), cssparser::ParseError<'_, ()>> =
        input.parse_nested_block(|input| {
            let r = input.expect_integer()?;
            input.expect_comma()?;
            let g = input.expect_integer()?;
            input.expect_comma()?;
            let b = input.expect_integer()?;
            Ok((r, g, b))
        });

    let (r, g, b) = result.map_err(|e| TcssError::Parse(format!("invalid rgb(): {e:?}")))?;

    let r = u8::try_from(r).map_err(|_| TcssError::InvalidValue {
        property: "color".into(),
        value: format!("rgb component {r} out of range"),
    })?;
    let g = u8::try_from(g).map_err(|_| TcssError::InvalidValue {
        property: "color".into(),
        value: format!("rgb component {g} out of range"),
    })?;
    let b = u8::try_from(b).map_err(|_| TcssError::InvalidValue {
        property: "color".into(),
        value: format!("rgb component {b} out of range"),
    })?;

    Ok(Color::Rgb { r, g, b })
}

/// Parse a length value from CSS input.
///
/// Supports:
/// - Bare numbers: `10` (interpreted as cells)
/// - Percentages: `50%`
/// - `auto` keyword
pub fn parse_length(input: &mut Parser<'_, '_>) -> Result<Length, TcssError> {
    let token = input
        .next()
        .map_err(|e| TcssError::Parse(format!("{e:?}")))?
        .clone();
    match &token {
        Token::Ident(name) if name.eq_ignore_ascii_case("auto") => Ok(Length::Auto),
        Token::Number {
            int_value: Some(v), ..
        } => {
            let val = u16::try_from(*v).map_err(|_| TcssError::InvalidValue {
                property: "length".into(),
                value: format!("{v} is out of range for cell count"),
            })?;
            Ok(Length::Cells(val))
        }
        Token::Number { value, .. } => {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let val = *value as u16;
            Ok(Length::Cells(val))
        }
        Token::Percentage { unit_value, .. } => Ok(Length::Percent(*unit_value * 100.0)),
        other => Err(TcssError::Parse(format!("expected length, got {other:?}"))),
    }
}

/// Parse an integer value from CSS input.
pub fn parse_integer(input: &mut Parser<'_, '_>) -> Result<i32, TcssError> {
    input
        .expect_integer()
        .map_err(|e| TcssError::Parse(format!("{e:?}")))
}

/// Parse a float value from CSS input.
pub fn parse_float(input: &mut Parser<'_, '_>) -> Result<f32, TcssError> {
    input
        .expect_number()
        .map_err(|e| TcssError::Parse(format!("{e:?}")))
}

/// Parse a keyword from CSS input.
pub fn parse_keyword(input: &mut Parser<'_, '_>) -> Result<String, TcssError> {
    input
        .expect_ident()
        .map(|s| s.to_string())
        .map_err(|e| TcssError::Parse(format!("{e:?}")))
}

/// Parse a property value given the property name.
///
/// Routes to the appropriate parser based on the property type.
pub fn parse_property_value(
    property: &PropertyName,
    input: &mut Parser<'_, '_>,
) -> Result<CssValue, TcssError> {
    match property {
        // Color properties
        PropertyName::Color | PropertyName::Background | PropertyName::BorderColor => {
            parse_color(input).map(CssValue::Color)
        }

        // Dimension properties
        PropertyName::Width
        | PropertyName::Height
        | PropertyName::MinWidth
        | PropertyName::MaxWidth
        | PropertyName::MinHeight
        | PropertyName::MaxHeight
        | PropertyName::FlexBasis
        | PropertyName::Gap => parse_length(input).map(CssValue::Length),

        // Box model (margin/padding) — length values
        PropertyName::Margin
        | PropertyName::MarginTop
        | PropertyName::MarginRight
        | PropertyName::MarginBottom
        | PropertyName::MarginLeft
        | PropertyName::Padding
        | PropertyName::PaddingTop
        | PropertyName::PaddingRight
        | PropertyName::PaddingBottom
        | PropertyName::PaddingLeft => parse_length(input).map(CssValue::Length),

        // Numeric properties
        PropertyName::FlexGrow | PropertyName::FlexShrink => {
            parse_integer(input).map(CssValue::Integer)
        }

        // Float properties
        PropertyName::Opacity => parse_float(input).map(CssValue::Float),

        // All keyword-based properties
        PropertyName::Display
        | PropertyName::FlexDirection
        | PropertyName::FlexWrap
        | PropertyName::JustifyContent
        | PropertyName::AlignItems
        | PropertyName::AlignSelf
        | PropertyName::Dock
        | PropertyName::Overflow
        | PropertyName::OverflowX
        | PropertyName::OverflowY
        | PropertyName::Visibility
        | PropertyName::TextAlign
        | PropertyName::ContentAlign
        | PropertyName::TextStyle
        | PropertyName::Border
        | PropertyName::BorderTop
        | PropertyName::BorderRight
        | PropertyName::BorderBottom
        | PropertyName::BorderLeft
        | PropertyName::GridTemplateColumns
        | PropertyName::GridTemplateRows
        | PropertyName::GridColumn
        | PropertyName::GridRow => parse_keyword(input).map(CssValue::Keyword),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;
    use cssparser::ParserInput;

    fn parse_with<T>(css: &str, f: impl FnOnce(&mut Parser<'_, '_>) -> T) -> T {
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);
        f(&mut parser)
    }

    #[test]
    fn parse_named_color() {
        let result = parse_with("red", parse_color);
        assert_eq!(result, Ok(Color::Named(NamedColor::Red)));
    }

    #[test]
    fn parse_named_color_blue() {
        let result = parse_with("blue", parse_color);
        assert_eq!(result, Ok(Color::Named(NamedColor::Blue)));
    }

    #[test]
    fn parse_hex_color_short() {
        let result = parse_with("#fff", parse_color);
        assert_eq!(
            result,
            Ok(Color::Rgb {
                r: 255,
                g: 255,
                b: 255
            })
        );
    }

    #[test]
    fn parse_hex_color_long() {
        let result = parse_with("#1e1e2e", parse_color);
        assert_eq!(
            result,
            Ok(Color::Rgb {
                r: 30,
                g: 30,
                b: 46
            })
        );
    }

    #[test]
    fn parse_rgb_function() {
        let result = parse_with("rgb(255, 0, 128)", parse_color);
        assert_eq!(
            result,
            Ok(Color::Rgb {
                r: 255,
                g: 0,
                b: 128
            })
        );
    }

    #[test]
    fn parse_invalid_color() {
        let result = parse_with("notacolor", parse_color);
        assert!(result.is_err());
    }

    #[test]
    fn parse_cell_length() {
        let result = parse_with("10", parse_length);
        assert_eq!(result, Ok(Length::Cells(10)));
    }

    #[test]
    fn parse_percent_length() {
        let result = parse_with("50%", parse_length);
        assert_eq!(result, Ok(Length::Percent(50.0)));
    }

    #[test]
    fn parse_auto_length() {
        let result = parse_with("auto", parse_length);
        assert_eq!(result, Ok(Length::Auto));
    }

    #[test]
    fn parse_integer_value() {
        let result = parse_with("42", parse_integer);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn parse_float_value() {
        let result = parse_with("0.5", parse_float);
        assert_eq!(result, Ok(0.5));
    }

    #[test]
    fn parse_keyword_value() {
        let result = parse_with("flex", parse_keyword);
        assert_eq!(result, Ok("flex".into()));
    }

    #[test]
    fn parse_property_color() {
        let result = parse_with("red", |p| parse_property_value(&PropertyName::Color, p));
        assert_eq!(
            result,
            Ok(CssValue::Color(Color::Named(NamedColor::Red)))
        );
    }

    #[test]
    fn parse_property_width() {
        let result = parse_with("20", |p| parse_property_value(&PropertyName::Width, p));
        assert_eq!(result, Ok(CssValue::Length(Length::Cells(20))));
    }

    #[test]
    fn parse_property_display() {
        let result = parse_with("flex", |p| parse_property_value(&PropertyName::Display, p));
        assert_eq!(result, Ok(CssValue::Keyword("flex".into())));
    }

    #[test]
    fn parse_property_flex_grow() {
        let result = parse_with("2", |p| parse_property_value(&PropertyName::FlexGrow, p));
        assert_eq!(result, Ok(CssValue::Integer(2)));
    }

    #[test]
    fn parse_property_opacity() {
        let result = parse_with("0.8", |p| parse_property_value(&PropertyName::Opacity, p));
        assert_eq!(result, Ok(CssValue::Float(0.8)));
    }

    #[test]
    fn parse_property_background_hex() {
        let result = parse_with("#ff0000", |p| {
            parse_property_value(&PropertyName::Background, p)
        });
        assert_eq!(
            result,
            Ok(CssValue::Color(Color::Rgb {
                r: 255,
                g: 0,
                b: 0
            }))
        );
    }

    #[test]
    fn parse_property_margin_percent() {
        let result = parse_with("25%", |p| parse_property_value(&PropertyName::Margin, p));
        assert_eq!(result, Ok(CssValue::Length(Length::Percent(25.0))));
    }
}
