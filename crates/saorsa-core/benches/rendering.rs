//! Rendering benchmarks — ScreenBuffer diff and segment rendering.

#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use saorsa_core::buffer::ScreenBuffer;
use saorsa_core::cell::Cell;
use saorsa_core::color::{Color, NamedColor};
use saorsa_core::geometry::Size;
use saorsa_core::segment::Segment;
use saorsa_core::style::Style;

/// Benchmark ScreenBuffer creation and diff for 80x24 grid.
fn benchmark_buffer_diff_80x24(c: &mut Criterion) {
    c.bench_function("buffer_diff_80x24", |b| {
        b.iter(|| {
            let previous = ScreenBuffer::new(Size::new(80, 24));
            let mut current = ScreenBuffer::new(Size::new(80, 24));
            // Write some cells
            let style = Style::new().fg(Color::Named(NamedColor::Red));
            for y in 0..24 {
                for x in 0..80 {
                    if (x + y) % 2 == 0 {
                        current.set(x, y, Cell::new("X", style.clone()));
                    }
                }
            }
            black_box(current.diff(&previous))
        })
    });
}

/// Benchmark ScreenBuffer creation and diff for 120x40 grid.
fn benchmark_buffer_diff_120x40(c: &mut Criterion) {
    c.bench_function("buffer_diff_120x40", |b| {
        b.iter(|| {
            let previous = ScreenBuffer::new(Size::new(120, 40));
            let mut current = ScreenBuffer::new(Size::new(120, 40));
            let style = Style::new().fg(Color::Named(NamedColor::Green));
            for y in 0..40 {
                for x in 0..120 {
                    if (x + y) % 3 == 0 {
                        current.set(x, y, Cell::new("O", style.clone()));
                    }
                }
            }
            black_box(current.diff(&previous))
        })
    });
}

/// Benchmark ScreenBuffer creation and diff for 200x60 grid.
fn benchmark_buffer_diff_200x60(c: &mut Criterion) {
    c.bench_function("buffer_diff_200x60", |b| {
        b.iter(|| {
            let previous = ScreenBuffer::new(Size::new(200, 60));
            let mut current = ScreenBuffer::new(Size::new(200, 60));
            let style = Style::new().fg(Color::Named(NamedColor::Blue)).bold(true);
            for y in 0..60 {
                for x in 0..200 {
                    if (x + y) % 4 == 0 {
                        current.set(x, y, Cell::new("*", style.clone()));
                    }
                }
            }
            black_box(current.diff(&previous))
        })
    });
}

/// Benchmark rendering 1000 styled segments to a buffer.
fn benchmark_segment_rendering_1000(c: &mut Criterion) {
    c.bench_function("segment_rendering_1000", |b| {
        let style = Style::new()
            .fg(Color::Rgb {
                r: 255,
                g: 100,
                b: 50,
            })
            .italic(true);
        let segments: Vec<Segment> = (0..1000)
            .map(|i| Segment::styled(format!("Segment{i}"), style.clone()))
            .collect();

        b.iter(|| {
            let mut buffer = ScreenBuffer::new(Size::new(80, 100));
            let mut x = 0_u16;
            let mut y = 0_u16;

            for seg in &segments {
                for grapheme in seg.text.chars() {
                    if x >= 80 {
                        x = 0;
                        y += 1;
                        if y >= 100 {
                            break;
                        }
                    }
                    let cell = Cell::new(grapheme.to_string(), seg.style.clone());
                    buffer.set(x, y, cell);
                    x += 1;
                }
            }
            black_box(buffer)
        })
    });
}

criterion_group!(
    benches,
    benchmark_buffer_diff_80x24,
    benchmark_buffer_diff_120x40,
    benchmark_buffer_diff_200x60,
    benchmark_segment_rendering_1000
);
criterion_main!(benches);
