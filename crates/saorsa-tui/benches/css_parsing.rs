//! CSS parsing benchmarks.

#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use saorsa_tui::tcss::parser::parse_stylesheet;

/// Benchmark parsing a simple CSS stylesheet (3-5 rules).
fn benchmark_parse_simple_stylesheet(c: &mut Criterion) {
    let css = r#"
        Label {
            color: red;
            background: #1e1e2e;
            width: 20;
        }

        Container {
            padding: 2;
            display: flex;
        }

        .error {
            color: red !important;
            text-style: bold;
        }
    "#;

    c.bench_function("parse_simple_stylesheet", |b| {
        b.iter(|| {
            let result = parse_stylesheet(black_box(css));
            assert!(result.is_ok());
            black_box(result)
        })
    });
}

/// Benchmark parsing a complex CSS stylesheet (20+ rules with variables).
fn benchmark_parse_complex_stylesheet(c: &mut Criterion) {
    let css = r#"
        :root {
            $primary: #89b4fa;
            $secondary: #f38ba8;
            $background: #1e1e2e;
            $surface: #313244;
            $text: #cdd6f4;
            $subtext: #a6adc8;
            $overlay: #6c7086;
            $muted: #45475a;
            $spacing: 2;
            $border-radius: 1;
        }

        Label {
            color: $text;
            background: transparent;
        }

        Container {
            background: $surface;
            padding: $spacing;
            border: solid;
            border-color: $overlay;
        }

        .error {
            color: $secondary;
            background: #f38ba833;
            text-style: bold;
        }

        .warning {
            color: yellow;
            background: #fab38733;
        }

        .success {
            color: green;
            background: #a6e3a133;
        }

        #sidebar {
            width: 30;
            background: $background;
            border-right: solid;
            border-color: $muted;
        }

        #main-content {
            flex-grow: 1;
            padding: $spacing;
            background: $background;
        }

        Container > Label.title {
            color: $primary;
            text-style: bold;
            margin-bottom: 1;
        }

        Container > Label.subtitle {
            color: $subtext;
            text-style: italic;
            margin-bottom: $spacing;
        }

        .button {
            color: $background;
            background: $primary;
            padding: 1;
            text-align: center;
            border: solid;
            border-color: $primary;
        }

        .button:hover {
            background: #74c7ec;
            border-color: #74c7ec;
        }

        .input {
            color: $text;
            background: $surface;
            padding: 1;
            border: solid;
            border-color: $overlay;
            width: 100%;
        }

        .input:focus {
            border-color: $primary;
        }

        .tab {
            color: $subtext;
            padding: 1;
            border-bottom: solid;
            border-color: transparent;
        }

        .tab.active {
            color: $primary;
            border-color: $primary;
            text-style: bold;
        }

        .card {
            background: $surface;
            padding: $spacing;
            border: solid;
            border-color: $muted;
            border-radius: $border-radius;
            margin: 1;
        }

        .list-item {
            color: $text;
            padding: 1;
            border-bottom: solid;
            border-color: $muted;
        }

        .list-item:hover {
            background: $overlay;
        }

        .list-item.selected {
            background: $primary;
            color: $background;
        }

        .header {
            background: $surface;
            padding: $spacing;
            border-bottom: solid;
            border-color: $muted;
        }

        .footer {
            background: $surface;
            padding: 1;
            border-top: solid;
            border-color: $muted;
            text-align: center;
            color: $subtext;
        }
    "#;

    c.bench_function("parse_complex_stylesheet", |b| {
        b.iter(|| {
            let result = parse_stylesheet(black_box(css));
            assert!(result.is_ok());
            black_box(result)
        })
    });
}

criterion_group!(
    benches,
    benchmark_parse_simple_stylesheet,
    benchmark_parse_complex_stylesheet
);
criterion_main!(benches);
