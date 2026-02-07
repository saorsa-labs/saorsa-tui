//! TCSS stylesheet parser.
//!
//! Parses TCSS stylesheets into the AST using the `cssparser` crate.

use cssparser::{Parser, ParserInput, Token};

use crate::color::Color;
use crate::tcss::ast::{Rule, Stylesheet};
use crate::tcss::error::TcssError;
use crate::tcss::property::{Declaration, PropertyName};
use crate::tcss::selector::SelectorList;
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

/// Parse a complete TCSS stylesheet from a string.
pub fn parse_stylesheet(input: &str) -> Result<Stylesheet, TcssError> {
    let mut parser_input = ParserInput::new(input);
    let mut parser = Parser::new(&mut parser_input);
    let mut stylesheet = Stylesheet::new();

    while !parser.is_exhausted() {
        match parse_rule(&mut parser) {
            Ok(rule) => stylesheet.add_rule(rule),
            Err(_) => {
                // Error recovery: skip to next block or end.
                // Try to skip past the next closing brace.
                let _ = skip_to_next_rule(&mut parser);
            }
        }
    }

    Ok(stylesheet)
}

/// Parse a single CSS rule: `selectors { declarations }`.
fn parse_rule(input: &mut Parser<'_, '_>) -> Result<Rule, TcssError> {
    // Parse selector list (everything before `{`).
    let selectors = SelectorList::parse_from(input)?;

    // Consume the `{` token to open the block.
    input
        .expect_curly_bracket_block()
        .map_err(|e| TcssError::Parse(format!("expected '{{': {e:?}")))?;

    // Parse declarations inside the block.
    let declarations: Result<Vec<Declaration>, cssparser::ParseError<'_, ()>> = input
        .parse_nested_block(|input| {
            let mut decls = Vec::new();

            while !input.is_exhausted() {
                match parse_declaration_inner(input) {
                    Ok(decl) => decls.push(decl),
                    Err(_) => {
                        // Skip to next semicolon or end of block.
                        while input.next().is_ok_and(|t| !matches!(t, Token::Semicolon)) {}
                    }
                }
            }

            Ok(decls)
        });

    let declarations = declarations.map_err(|e| TcssError::Parse(format!("{e:?}")))?;

    Ok(Rule::new(selectors, declarations))
}

/// Parse a single declaration inside a rule block.
///
/// Expected format: `property-name: value [!important] ;`
fn parse_declaration_inner<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Declaration, cssparser::ParseError<'i, ()>> {
    // Parse property name (an ident token).
    let name = input.expect_ident()?.to_string();

    // Expect colon.
    input.expect_colon()?;

    // Look up the property name.
    let property = PropertyName::from_css(&name).ok_or_else(|| input.new_custom_error(()))?;

    // Parse the value using our typed parser.
    let value = parse_property_value(&property, input).map_err(|_| input.new_custom_error(()))?;

    // Check for !important.
    let important = input
        .try_parse(|p| -> Result<(), cssparser::ParseError<'_, ()>> {
            p.expect_delim('!')?;
            p.expect_ident_matching("important")?;
            Ok(())
        })
        .is_ok();

    // Consume optional semicolon.
    let _ = input.try_parse(|p| p.expect_semicolon());

    Ok(Declaration {
        property,
        value,
        important,
    })
}

/// Skip to the next rule start (after a `}` or past unknown content).
fn skip_to_next_rule(input: &mut Parser<'_, '_>) -> Result<(), ()> {
    // Try parsing a curly block and discarding it.
    if input.expect_curly_bracket_block().is_ok() {
        let _ = input.parse_nested_block(|input| -> Result<(), cssparser::ParseError<'_, ()>> {
            while input.next().is_ok() {}
            Ok(())
        });
        return Ok(());
    }
    // Otherwise skip a token.
    if input.next().is_ok() {
        Ok(())
    } else {
        Err(())
    }
}

/// Parse a single CSS declaration from a string.
///
/// Convenience function for parsing isolated declarations like `color: red`.
pub fn parse_declaration(input_str: &str) -> Result<Declaration, TcssError> {
    let mut parser_input = ParserInput::new(input_str);
    let mut parser = Parser::new(&mut parser_input);

    let result: Result<Declaration, cssparser::ParseError<'_, ()>> =
        parse_declaration_inner(&mut parser);

    result.map_err(|e| TcssError::Parse(format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;

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
        assert_eq!(result, Ok(CssValue::Color(Color::Named(NamedColor::Red))));
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
            Ok(CssValue::Color(Color::Rgb { r: 255, g: 0, b: 0 }))
        );
    }

    #[test]
    fn parse_property_margin_percent() {
        let result = parse_with("25%", |p| parse_property_value(&PropertyName::Margin, p));
        assert_eq!(result, Ok(CssValue::Length(Length::Percent(25.0))));
    }

    // --- Stylesheet parser tests (Task 7) ---

    /// Parse a stylesheet, asserting success and returning the result.
    fn parse_sheet(css: &str) -> Stylesheet {
        let result = parse_stylesheet(css);
        assert!(
            result.is_ok(),
            "parse_stylesheet failed for input: {result:?}"
        );
        match result {
            Ok(sheet) => sheet,
            Err(_) => unreachable!(),
        }
    }

    /// Parse a declaration, asserting success and returning the result.
    fn parse_decl(css: &str) -> Declaration {
        let result = parse_declaration(css);
        assert!(
            result.is_ok(),
            "parse_declaration failed for input: {result:?}"
        );
        match result {
            Ok(decl) => decl,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn parse_empty_stylesheet() {
        let sheet = parse_sheet("");
        assert!(sheet.is_empty());
    }

    #[test]
    fn parse_whitespace_only_stylesheet() {
        let sheet = parse_sheet("   \n\t  ");
        assert!(sheet.is_empty());
    }

    #[test]
    fn parse_single_rule() {
        let sheet = parse_sheet("Label { color: red; }");
        assert_eq!(sheet.len(), 1);
        let rule = &sheet.rules()[0];
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, PropertyName::Color);
        assert_eq!(
            rule.declarations[0].value,
            CssValue::Color(Color::Named(NamedColor::Red))
        );
    }

    #[test]
    fn parse_multiple_declarations() {
        let sheet = parse_sheet("Label { color: red; background: blue; }");
        assert_eq!(sheet.len(), 1);
        let rule = &sheet.rules()[0];
        assert_eq!(rule.declarations.len(), 2);
        assert_eq!(rule.declarations[0].property, PropertyName::Color);
        assert_eq!(rule.declarations[1].property, PropertyName::Background);
    }

    #[test]
    fn parse_multiple_rules() {
        let sheet = parse_sheet("Label { color: red; } Container { background: blue; }");
        assert_eq!(sheet.len(), 2);
    }

    #[test]
    fn parse_important_declaration() {
        let sheet = parse_sheet("Label { color: red !important; }");
        assert_eq!(sheet.len(), 1);
        assert!(sheet.rules()[0].declarations[0].important);
    }

    #[test]
    fn parse_with_comments() {
        let sheet = parse_sheet("/* theme */ Label { color: red; }");
        assert_eq!(sheet.len(), 1);
    }

    #[test]
    fn parse_complex_selector_rule() {
        let sheet = parse_sheet("Container > Label { color: red; }");
        assert_eq!(sheet.len(), 1);
        let selector = &sheet.rules()[0].selectors.selectors[0];
        assert!(!selector.chain.is_empty());
    }

    #[test]
    fn parse_selector_list_rule() {
        let sheet = parse_sheet("Label, Container { color: red; }");
        assert_eq!(sheet.len(), 1);
        assert_eq!(sheet.rules()[0].selectors.selectors.len(), 2);
    }

    #[test]
    fn parse_all_property_types() {
        let css = r#"
            Label {
                color: red;
                width: 20;
                display: flex;
                flex-grow: 2;
                opacity: 0.5;
            }
        "#;
        let sheet = parse_sheet(css);
        assert_eq!(sheet.len(), 1);
        let decls = &sheet.rules()[0].declarations;
        assert_eq!(decls.len(), 5);
        assert!(matches!(decls[0].value, CssValue::Color(_)));
        assert!(matches!(decls[1].value, CssValue::Length(_)));
        assert!(matches!(decls[2].value, CssValue::Keyword(_)));
        assert!(matches!(decls[3].value, CssValue::Integer(_)));
        assert!(matches!(decls[4].value, CssValue::Float(_)));
    }

    #[test]
    fn parse_invalid_rule_skipped() {
        let sheet = parse_sheet("!!! invalid { } Label { color: red; }");
        // At least one valid rule should be parsed
        assert!(!sheet.is_empty());
    }

    #[test]
    fn parse_real_world_stylesheet() {
        let css = r#"
            /* Base styles */
            Label {
                color: white;
            }

            Container {
                background: #1e1e2e;
                padding: 1;
            }

            .error {
                color: red;
            }

            #sidebar {
                width: 30;
                background: #2e2e3e;
            }

            Container > Label.title {
                color: blue;
                text-style: bold;
            }
        "#;
        let sheet = parse_sheet(css);
        assert_eq!(sheet.len(), 5);
    }

    #[test]
    fn parse_declaration_standalone() {
        let decl = parse_decl("color: red");
        assert_eq!(decl.property, PropertyName::Color);
        assert_eq!(decl.value, CssValue::Color(Color::Named(NamedColor::Red)));
        assert!(!decl.important);
    }

    #[test]
    fn parse_declaration_important_standalone() {
        let decl = parse_decl("color: red !important");
        assert_eq!(decl.property, PropertyName::Color);
        assert!(decl.important);
    }
}
