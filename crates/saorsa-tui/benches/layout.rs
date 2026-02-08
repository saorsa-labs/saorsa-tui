//! Layout engine benchmarks — Taffy layout computation.

#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use saorsa_tui::layout::engine::LayoutEngine;
use taffy::prelude::*;

/// Benchmark Taffy layout computation for 10-node flexbox tree.
fn benchmark_layout_10_nodes(c: &mut Criterion) {
    c.bench_function("layout_10_nodes", |b| {
        b.iter(|| {
            let mut engine = LayoutEngine::new();
            // Create 10 leaf nodes
            for i in 0..10 {
                let result = engine.add_node(
                    i,
                    Style {
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                );
                assert!(result.is_ok());
            }
            // Root container
            let result = engine.add_node_with_children(
                100,
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(800.0),
                        height: Dimension::Length(600.0),
                    },
                    ..Default::default()
                },
                &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            );
            assert!(result.is_ok());
            let result = engine.set_root(100);
            assert!(result.is_ok());
            let result = engine.compute(800, 600);
            assert!(result.is_ok());
            black_box(engine)
        })
    });
}

/// Benchmark Taffy layout computation for 50-node flexbox tree.
fn benchmark_layout_50_nodes(c: &mut Criterion) {
    c.bench_function("layout_50_nodes", |b| {
        b.iter(|| {
            let mut engine = LayoutEngine::new();
            // Create 50 leaf nodes
            for i in 0..50 {
                let result = engine.add_node(
                    i,
                    Style {
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                );
                assert!(result.is_ok());
            }
            // Root container
            let children: Vec<u64> = (0..50).collect();
            let result = engine.add_node_with_children(
                100,
                Style {
                    flex_wrap: FlexWrap::Wrap,
                    size: taffy::Size {
                        width: Dimension::Length(800.0),
                        height: Dimension::Length(600.0),
                    },
                    ..Default::default()
                },
                &children,
            );
            assert!(result.is_ok());
            let result = engine.set_root(100);
            assert!(result.is_ok());
            let result = engine.compute(800, 600);
            assert!(result.is_ok());
            black_box(engine)
        })
    });
}

/// Benchmark Taffy layout computation for 100-node flexbox tree.
fn benchmark_layout_100_nodes(c: &mut Criterion) {
    c.bench_function("layout_100_nodes", |b| {
        b.iter(|| {
            let mut engine = LayoutEngine::new();
            // Create 100 leaf nodes with varying sizes
            for i in 0..100 {
                let result = engine.add_node(
                    i,
                    Style {
                        flex_grow: if i % 3 == 0 { 2.0 } else { 1.0 },
                        size: taffy::Size {
                            width: if i % 5 == 0 {
                                Dimension::Length(50.0)
                            } else {
                                auto()
                            },
                            height: auto(),
                        },
                        ..Default::default()
                    },
                );
                assert!(result.is_ok());
            }
            // Root container
            let children: Vec<u64> = (0..100).collect();
            let result = engine.add_node_with_children(
                200,
                Style {
                    flex_wrap: FlexWrap::Wrap,
                    size: taffy::Size {
                        width: Dimension::Length(1200.0),
                        height: Dimension::Length(800.0),
                    },
                    ..Default::default()
                },
                &children,
            );
            assert!(result.is_ok());
            let result = engine.set_root(200);
            assert!(result.is_ok());
            let result = engine.compute(1200, 800);
            assert!(result.is_ok());
            black_box(engine)
        })
    });
}

criterion_group!(
    benches,
    benchmark_layout_10_nodes,
    benchmark_layout_50_nodes,
    benchmark_layout_100_nodes
);
criterion_main!(benches);
