//! Apply TCSS computed styles to concrete widgets.

use crate::Color;
use crate::color::NamedColor;
use crate::style::Style;
use crate::tcss::ComputedStyle;
use crate::tcss::property::PropertyName;
use crate::tcss::value::{CssValue, Length};
use crate::widget::{Alignment, BorderStyle, Container, Label, RichLog};

/// Widgets that can accept a [`ComputedStyle`] at runtime.
pub trait ApplyComputedStyle {
    /// Apply a computed style to the widget.
    fn apply_computed_style(&mut self, computed: &ComputedStyle);
}

fn css_color(v: &CssValue) -> Option<Color> {
    match v {
        CssValue::Color(c) => Some(c.clone()),
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "black" => Some(Color::Named(NamedColor::Black)),
            "red" => Some(Color::Named(NamedColor::Red)),
            "green" => Some(Color::Named(NamedColor::Green)),
            "yellow" => Some(Color::Named(NamedColor::Yellow)),
            "blue" => Some(Color::Named(NamedColor::Blue)),
            "magenta" => Some(Color::Named(NamedColor::Magenta)),
            "cyan" => Some(Color::Named(NamedColor::Cyan)),
            "white" => Some(Color::Named(NamedColor::White)),
            _ => None,
        },
        _ => None,
    }
}

fn css_cells(v: &CssValue) -> Option<u16> {
    match v {
        CssValue::Length(Length::Cells(n)) => Some(*n),
        CssValue::Integer(n) if *n >= 0 => Some(*n as u16),
        _ => None,
    }
}

fn apply_text_style(style: &mut Style, v: &CssValue) {
    let CssValue::Keyword(k) = v else {
        return;
    };
    // Allow future multi-token values by splitting on whitespace.
    for token in k.split_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "bold" => style.bold = true,
            "italic" => style.italic = true,
            "underline" => style.underline = true,
            "strikethrough" | "strike" => style.strikethrough = true,
            "dim" => style.dim = true,
            "reverse" => style.reverse = true,
            _ => {}
        }
    }
}

fn apply_opacity(style: &mut Style, v: &CssValue) {
    let CssValue::Float(f) = v else {
        return;
    };
    if *f < 0.999 {
        style.dim = true;
    }
}

fn apply_text_align(current: &mut Alignment, v: &CssValue) {
    let CssValue::Keyword(k) = v else {
        return;
    };
    *current = match k.to_ascii_lowercase().as_str() {
        "center" => Alignment::Center,
        "right" | "end" => Alignment::Right,
        _ => Alignment::Left,
    };
}

fn apply_border_style(current: &mut BorderStyle, v: &CssValue) {
    let CssValue::Keyword(k) = v else {
        return;
    };
    *current = match k.to_ascii_lowercase().as_str() {
        "none" => BorderStyle::None,
        "double" => BorderStyle::Double,
        "heavy" | "thick" => BorderStyle::Heavy,
        "round" | "rounded" => BorderStyle::Rounded,
        "single" | "ascii" | "box" => BorderStyle::Single,
        _ => *current,
    };
}

impl ApplyComputedStyle for Label {
    fn apply_computed_style(&mut self, computed: &ComputedStyle) {
        let mut style = Style::default();

        if let Some(v) = computed.get(&PropertyName::Color)
            && let Some(c) = css_color(v)
        {
            style.fg = Some(c);
        }
        if let Some(v) = computed.get(&PropertyName::Background)
            && let Some(c) = css_color(v)
        {
            style.bg = Some(c);
        }
        if let Some(v) = computed.get(&PropertyName::TextStyle) {
            apply_text_style(&mut style, v);
        }
        if let Some(v) = computed.get(&PropertyName::Opacity) {
            apply_opacity(&mut style, v);
        }

        self.set_style(style);

        if let Some(v) = computed.get(&PropertyName::TextAlign) {
            let mut a = self.alignment_value();
            apply_text_align(&mut a, v);
            self.set_alignment(a);
        }
    }
}

impl ApplyComputedStyle for Container {
    fn apply_computed_style(&mut self, computed: &ComputedStyle) {
        // Border style.
        if let Some(v) = computed.get(&PropertyName::Border) {
            let mut b = self.border_style_kind();
            apply_border_style(&mut b, v);
            self.set_border(b);
        }

        // Border color.
        if let Some(v) = computed.get(&PropertyName::BorderColor)
            && let Some(c) = css_color(v)
        {
            let s = Style::default().fg(c);
            self.set_border_style(s.clone());
            self.set_title_style(s);
        }

        // Background.
        if let Some(v) = computed.get(&PropertyName::Background)
            && let Some(c) = css_color(v)
        {
            self.set_fill_style(Style::default().bg(c));
        } else {
            self.set_fill_style(Style::default());
        }

        // Padding (cells only).
        if let Some(v) = computed.get(&PropertyName::Padding)
            && let Some(p) = css_cells(v)
        {
            self.set_padding(p);
        }
    }
}

impl ApplyComputedStyle for RichLog {
    fn apply_computed_style(&mut self, computed: &ComputedStyle) {
        let mut style = Style::default();
        if let Some(v) = computed.get(&PropertyName::Color)
            && let Some(c) = css_color(v)
        {
            style.fg = Some(c);
        }
        if let Some(v) = computed.get(&PropertyName::Background)
            && let Some(c) = css_color(v)
        {
            style.bg = Some(c);
        }
        if let Some(v) = computed.get(&PropertyName::TextStyle) {
            apply_text_style(&mut style, v);
        }
        if let Some(v) = computed.get(&PropertyName::Opacity) {
            apply_opacity(&mut style, v);
        }
        self.set_base_style(style);

        if let Some(v) = computed.get(&PropertyName::Border) {
            let mut b = self.border_style_kind();
            apply_border_style(&mut b, v);
            self.set_border(b);
        }
    }
}
