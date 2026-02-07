//! CSS value types for TCSS properties.

use crate::color::Color;

/// A CSS length value.
#[derive(Clone, Debug, PartialEq)]
pub enum Length {
    /// Fixed cell count.
    Cells(u16),
    /// Percentage of parent dimension.
    Percent(f32),
    /// Auto sizing.
    Auto,
}

/// A CSS value used in declarations.
#[derive(Clone, Debug, PartialEq)]
pub enum CssValue {
    /// A color value.
    Color(Color),
    /// A length value.
    Length(Length),
    /// A keyword (e.g., "bold", "flex", "center").
    Keyword(String),
    /// An integer (e.g., flex-grow: 2).
    Integer(i32),
    /// A float (e.g., opacity: 0.5).
    Float(f32),
    /// A fractional unit for grid layout (e.g., 1fr).
    Fr(f32),
    /// A string value.
    String(String),
    /// A variable reference ($name), resolved during cascade.
    Variable(String),
    /// An ordered list of values (e.g., grid-template-columns: 1fr 2fr 100).
    List(Vec<CssValue>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;

    #[test]
    fn length_cells() {
        let len = Length::Cells(10);
        assert_eq!(len, Length::Cells(10));
    }

    #[test]
    fn length_percent() {
        let len = Length::Percent(50.0);
        assert_eq!(len, Length::Percent(50.0));
    }

    #[test]
    fn length_auto() {
        let len = Length::Auto;
        assert_eq!(len, Length::Auto);
    }

    #[test]
    fn length_clone() {
        let len = Length::Cells(5);
        let len2 = len.clone();
        assert_eq!(len, len2);
    }

    #[test]
    fn value_color() {
        let val = CssValue::Color(Color::Named(NamedColor::Red));
        assert!(matches!(val, CssValue::Color(_)));
    }

    #[test]
    fn value_length() {
        let val = CssValue::Length(Length::Cells(20));
        assert!(matches!(val, CssValue::Length(Length::Cells(20))));
    }

    #[test]
    fn value_keyword() {
        let val = CssValue::Keyword("bold".into());
        assert_eq!(val, CssValue::Keyword("bold".into()));
    }

    #[test]
    fn value_integer() {
        let val = CssValue::Integer(2);
        assert_eq!(val, CssValue::Integer(2));
    }

    #[test]
    fn value_float() {
        let val = CssValue::Float(0.5);
        assert_eq!(val, CssValue::Float(0.5));
    }

    #[test]
    fn value_fr() {
        let val = CssValue::Fr(1.0);
        assert_eq!(val, CssValue::Fr(1.0));
    }

    #[test]
    fn value_string() {
        let val = CssValue::String("hello".into());
        assert_eq!(val, CssValue::String("hello".into()));
    }

    #[test]
    fn value_clone() {
        let val = CssValue::Integer(42);
        let val2 = val.clone();
        assert_eq!(val, val2);
    }

    #[test]
    fn value_variable() {
        let val = CssValue::Variable("primary".into());
        assert_eq!(val, CssValue::Variable("primary".into()));
    }

    #[test]
    fn value_variable_clone_and_eq() {
        let val = CssValue::Variable("fg-color".into());
        let val2 = val.clone();
        assert_eq!(val, val2);
    }

    #[test]
    fn value_list_empty() {
        let val = CssValue::List(vec![]);
        assert_eq!(val, CssValue::List(vec![]));
    }

    #[test]
    fn value_list_single() {
        let val = CssValue::List(vec![CssValue::Fr(1.0)]);
        assert!(matches!(val, CssValue::List(v) if v.len() == 1));
    }

    #[test]
    fn value_list_multiple() {
        let val = CssValue::List(vec![
            CssValue::Fr(1.0),
            CssValue::Fr(2.0),
            CssValue::Length(Length::Cells(100)),
        ]);
        assert!(matches!(val, CssValue::List(v) if v.len() == 3));
    }

    #[test]
    fn value_list_clone() {
        let val = CssValue::List(vec![
            CssValue::Fr(1.0),
            CssValue::Length(Length::Percent(50.0)),
        ]);
        let val2 = val.clone();
        assert_eq!(val, val2);
    }

    #[test]
    fn value_list_eq() {
        let a = CssValue::List(vec![CssValue::Fr(1.0), CssValue::Fr(2.0)]);
        let b = CssValue::List(vec![CssValue::Fr(1.0), CssValue::Fr(2.0)]);
        assert_eq!(a, b);
    }

    #[test]
    fn value_list_nested() {
        let inner = CssValue::List(vec![CssValue::Integer(1), CssValue::Integer(2)]);
        let outer = CssValue::List(vec![inner.clone(), CssValue::Fr(1.0)]);
        assert!(matches!(outer, CssValue::List(v) if v.len() == 2));
    }

    #[test]
    fn value_list_mixed_types() {
        let val = CssValue::List(vec![
            CssValue::Fr(1.0),
            CssValue::Length(Length::Cells(50)),
            CssValue::Keyword("auto".into()),
        ]);
        assert!(matches!(val, CssValue::List(v) if v.len() == 3));
    }

    #[test]
    fn value_fr_clone_and_eq() {
        let val = CssValue::Fr(2.5);
        let val2 = val.clone();
        assert_eq!(val, val2);
    }
}
