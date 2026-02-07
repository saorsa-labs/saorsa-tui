//! Property-based tests for the TCSS parser.
//!
//! Uses proptest to generate random CSS inputs and verify the parser
//! handles them without panicking.

use cssparser::{Parser, ParserInput};
use proptest::prelude::*;
use saorsa_core::color::{Color, NamedColor};
use saorsa_core::tcss::parser::{parse_color, parse_keyword, parse_length, parse_property_value};
use saorsa_core::tcss::property::PropertyName;
use saorsa_core::tcss::selector::SelectorList;
use saorsa_core::tcss::value::{CssValue, Length};
use saorsa_core::tcss::variable::{VariableEnvironment, VariableMap};

// ==============================================================================
// Property Test 1: Random hex color strings don't panic the parser
// ==============================================================================

proptest! {
    #[test]
    fn random_hex_colors_dont_panic(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let hex_string = format!("#{:02x}{:02x}{:02x}", r, g, b);
        let mut input = ParserInput::new(&hex_string);
        let mut parser = Parser::new(&mut input);

        // Parser should either succeed or return an error, but never panic.
        let _result = parse_color(&mut parser);
    }
}

// ==============================================================================
// Property Test 2: Random length values (number + unit) don't panic
// ==============================================================================

proptest! {
    #[test]
    fn random_lengths_dont_panic(value in 0u16..=1000) {
        let length_string = format!("{value}");
        let mut input = ParserInput::new(&length_string);
        let mut parser = Parser::new(&mut input);

        // Parser should handle any positive integer as a cell count.
        let _result = parse_length(&mut parser);
    }
}

// ==============================================================================
// Property Test 3: Random percentage values don't panic
// ==============================================================================

proptest! {
    #[test]
    fn random_percentages_dont_panic(value in 0.0f32..=100.0) {
        let pct_string = format!("{value}%");
        let mut input = ParserInput::new(&pct_string);
        let mut parser = Parser::new(&mut input);

        // Parser should handle any percentage value.
        let _result = parse_length(&mut parser);
    }
}

// ==============================================================================
// Property Test 4: Random keyword strings don't panic
// ==============================================================================

proptest! {
    #[test]
    fn random_keywords_dont_panic(s in "[a-z]{1,20}") {
        let mut input = ParserInput::new(&s);
        let mut parser = Parser::new(&mut input);

        // Parser should either return the keyword or an error, but never panic.
        let _result = parse_keyword(&mut parser);
    }
}

// ==============================================================================
// Property Test 5: Random selector strings don't panic
// ==============================================================================

proptest! {
    #[test]
    fn random_selectors_dont_panic(s in "[a-zA-Z]{1,10}") {
        // Parser should either parse a selector or return an error, never panic.
        let _result = SelectorList::parse(&s);
    }
}

// ==============================================================================
// Property Test 6: Variable resolution with random names returns value or None
// ==============================================================================

proptest! {
    #[test]
    fn random_variable_names_resolve_or_none(name in "[a-z]{1,20}") {
        let env = VariableEnvironment::new();

        // Resolving a non-existent variable should return None, not panic.
        let result = env.resolve(&name);
        assert!(result.is_none());
    }
}

// ==============================================================================
// Property Test 7: Round-trip hex color parsing
// ==============================================================================

proptest! {
    #[test]
    fn roundtrip_hex_color(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let hex_string = format!("#{:02x}{:02x}{:02x}", r, g, b);
        let mut input = ParserInput::new(&hex_string);
        let mut parser = Parser::new(&mut input);

        // Parse the hex color.
        let result = parse_color(&mut parser);
        assert!(result.is_ok());

        let color = match result {
            Ok(c) => c,
            Err(_) => {
                unreachable!();
            }
        };

        // Verify it parsed as RGB with the expected values.
        match color {
            Color::Rgb { r: pr, g: pg, b: pb } => {
                assert_eq!(pr, r);
                assert_eq!(pg, g);
                assert_eq!(pb, b);
            }
            _ => {
                unreachable!();
            }
        }
    }
}

// ==============================================================================
// Property Test 8: Random CSS property declaration strings don't panic
// ==============================================================================

proptest! {
    #[test]
    fn random_property_values_dont_panic(value in 0i32..=1000) {
        let value_string = format!("{value}");
        let mut input = ParserInput::new(&value_string);
        let mut parser = Parser::new(&mut input);

        // Try to parse as a flex-grow property (expects integer).
        let _result = parse_property_value(&PropertyName::FlexGrow, &mut parser);
    }
}

// ==============================================================================
// Property Test 9: Random variable definition strings don't panic
// ==============================================================================

proptest! {
    #[test]
    fn random_variable_definitions_dont_panic(name in "[a-z]{1,20}") {
        let mut var_map = VariableMap::new();

        // Setting a variable with a random name should never panic.
        var_map.set(&name, CssValue::Color(Color::Named(NamedColor::Red)));

        // Getting it back should return the value.
        let result = var_map.get(&name);
        assert!(result.is_some());
    }
}

// ==============================================================================
// Property Test 10: Random named colors resolve correctly
// ==============================================================================

proptest! {
    #[test]
    fn random_named_colors_parse_or_error(color_name in "(red|blue|green|white|black|yellow)") {
        let mut input = ParserInput::new(&color_name);
        let mut parser = Parser::new(&mut input);

        // Named colors should parse successfully.
        let result = parse_color(&mut parser);
        assert!(result.is_ok());

        let color = match result {
            Ok(c) => c,
            Err(_) => {
                unreachable!();
            }
        };

        // Verify it's a named color.
        assert!(matches!(color, Color::Named(_)));
    }
}

// ==============================================================================
// Property Test 11: Length auto keyword round-trip
// ==============================================================================

#[test]
fn length_auto_roundtrip() {
    let input_str = "auto";
    let mut input = ParserInput::new(input_str);
    let mut parser = Parser::new(&mut input);

    let result = parse_length(&mut parser);
    assert!(result.is_ok());

    let length = match result {
        Ok(l) => l,
        Err(_) => {
            unreachable!();
        }
    };

    assert_eq!(length, Length::Auto);
}

// ==============================================================================
// Property Test 12: Variable reference parsing with random names
// ==============================================================================

proptest! {
    #[test]
    fn random_variable_references_parse(name in "[a-z]{1,20}") {
        let var_string = format!("${name}");
        let mut input = ParserInput::new(&var_string);
        let mut parser = Parser::new(&mut input);

        // Try to parse as a color property (which accepts variables).
        let result = parse_property_value(&PropertyName::Color, &mut parser);

        // Should parse as a variable reference.
        match result {
            Ok(CssValue::Variable(parsed_name)) => {
                assert_eq!(parsed_name, name);
            }
            _ => {
                unreachable!();
            }
        }
    }
}
