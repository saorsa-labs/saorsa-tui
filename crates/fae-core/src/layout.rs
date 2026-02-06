//! Layout system for splitting terminal areas.

use crate::geometry::Rect;

/// Direction of layout splitting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Stack children top to bottom.
    Vertical,
    /// Stack children left to right.
    Horizontal,
}

/// Constraint for a layout segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constraint {
    /// Fixed size in cells.
    Fixed(u16),
    /// Minimum size in cells.
    Min(u16),
    /// Maximum size in cells.
    Max(u16),
    /// Percentage of available space (0-100).
    Percentage(u8),
    /// Fill remaining space (distributed equally among all Fill constraints).
    Fill,
}

/// Dock position for anchoring a widget to an edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dock {
    /// Dock to the top edge.
    Top,
    /// Dock to the bottom edge.
    Bottom,
    /// Dock to the left edge.
    Left,
    /// Dock to the right edge.
    Right,
}

/// Layout utilities for splitting terminal areas.
pub struct Layout;

impl Layout {
    /// Split an area into segments along the given direction using constraints.
    ///
    /// Returns a `Vec<Rect>` with one rect per constraint.
    pub fn split(area: Rect, direction: Direction, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return Vec::new();
        }

        let total = match direction {
            Direction::Vertical => area.size.height,
            Direction::Horizontal => area.size.width,
        };

        let sizes = solve_constraints(total, constraints);

        let mut results = Vec::with_capacity(constraints.len());
        let mut offset: u16 = 0;

        for &size in &sizes {
            let rect = match direction {
                Direction::Vertical => Rect::new(area.position.x, area.position.y + offset, area.size.width, size),
                Direction::Horizontal => Rect::new(area.position.x + offset, area.position.y, size, area.size.height),
            };
            results.push(rect);
            offset = offset.saturating_add(size);
        }

        results
    }

    /// Dock a region to one edge of the area.
    ///
    /// Returns `(docked_rect, remaining_rect)`.
    pub fn dock(area: Rect, dock: Dock, size: u16) -> (Rect, Rect) {
        match dock {
            Dock::Top => {
                let s = size.min(area.size.height);
                (
                    Rect::new(area.position.x, area.position.y, area.size.width, s),
                    Rect::new(
                        area.position.x,
                        area.position.y + s,
                        area.size.width,
                        area.size.height.saturating_sub(s),
                    ),
                )
            }
            Dock::Bottom => {
                let s = size.min(area.size.height);
                (
                    Rect::new(
                        area.position.x,
                        area.position.y + area.size.height.saturating_sub(s),
                        area.size.width,
                        s,
                    ),
                    Rect::new(
                        area.position.x,
                        area.position.y,
                        area.size.width,
                        area.size.height.saturating_sub(s),
                    ),
                )
            }
            Dock::Left => {
                let s = size.min(area.size.width);
                (
                    Rect::new(area.position.x, area.position.y, s, area.size.height),
                    Rect::new(
                        area.position.x + s,
                        area.position.y,
                        area.size.width.saturating_sub(s),
                        area.size.height,
                    ),
                )
            }
            Dock::Right => {
                let s = size.min(area.size.width);
                (
                    Rect::new(
                        area.position.x + area.size.width.saturating_sub(s),
                        area.position.y,
                        s,
                        area.size.height,
                    ),
                    Rect::new(
                        area.position.x,
                        area.position.y,
                        area.size.width.saturating_sub(s),
                        area.size.height,
                    ),
                )
            }
        }
    }
}

/// Solve constraints to produce sizes that fit within `total`.
fn solve_constraints(total: u16, constraints: &[Constraint]) -> Vec<u16> {
    let n = constraints.len();
    let mut sizes = vec![0u16; n];
    let mut remaining = total;

    // Pass 1: allocate Fixed constraints
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Fixed(s) = c {
            let s = (*s).min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 2: allocate Percentage constraints
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Percentage(p) = c {
            let s = ((u32::from(total) * u32::from(*p)) / 100) as u16;
            let s = s.min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 3: allocate Min constraints (give at least min, but not more than remaining for now)
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Min(min) = c {
            let s = (*min).min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 4: allocate Max constraints
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Max(max) = c {
            let s = (*max).min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 5: distribute remaining among Fill constraints
    let fill_count = constraints
        .iter()
        .filter(|c| matches!(c, Constraint::Fill))
        .count();
    if fill_count > 0 {
        let each = remaining / fill_count as u16;
        let mut extra = remaining % fill_count as u16;
        for (i, c) in constraints.iter().enumerate() {
            if matches!(c, Constraint::Fill) {
                let bonus = if extra > 0 {
                    extra -= 1;
                    1
                } else {
                    0
                };
                sizes[i] = each + bonus;
            }
        }
    }

    sizes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn vertical_split_fixed() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fixed(3), Constraint::Fixed(5)],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 80, 3));
        assert_eq!(rects[1], Rect::new(0, 3, 80, 5));
    }

    #[test]
    fn horizontal_split_fixed() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Horizontal,
            &[Constraint::Fixed(20), Constraint::Fixed(30)],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 20, 24));
        assert_eq!(rects[1], Rect::new(20, 0, 30, 24));
    }

    #[test]
    fn vertical_fixed_plus_fill() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fixed(3), Constraint::Fill],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 80, 3));
        assert_eq!(rects[1], Rect::new(0, 3, 80, 21));
    }

    #[test]
    fn multiple_fills_distribute_equally() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fill, Constraint::Fill],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].size.height, 12);
        assert_eq!(rects[1].size.height, 12);
    }

    #[test]
    fn percentage_split() {
        let area = Rect::new(0, 0, 100, 10);
        let rects = Layout::split(
            area,
            Direction::Horizontal,
            &[Constraint::Percentage(30), Constraint::Percentage(70)],
        );
        assert_eq!(rects[0].size.width, 30);
        assert_eq!(rects[1].size.width, 70);
    }

    #[test]
    fn empty_constraints() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(area, Direction::Vertical, &[]);
        assert!(rects.is_empty());
    }

    #[test]
    fn dock_top() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Top, 3);
        assert_eq!(docked, Rect::new(0, 0, 80, 3));
        assert_eq!(remaining, Rect::new(0, 3, 80, 21));
    }

    #[test]
    fn dock_bottom() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Bottom, 3);
        assert_eq!(docked, Rect::new(0, 21, 80, 3));
        assert_eq!(remaining, Rect::new(0, 0, 80, 21));
    }

    #[test]
    fn dock_left() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Left, 20);
        assert_eq!(docked, Rect::new(0, 0, 20, 24));
        assert_eq!(remaining, Rect::new(20, 0, 60, 24));
    }

    #[test]
    fn dock_right() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Right, 20);
        assert_eq!(docked, Rect::new(60, 0, 20, 24));
        assert_eq!(remaining, Rect::new(0, 0, 60, 24));
    }

    #[test]
    fn dock_larger_than_area() {
        let area = Rect::new(0, 0, 80, 10);
        let (docked, remaining) = Layout::dock(area, Dock::Top, 20);
        assert_eq!(docked, Rect::new(0, 0, 80, 10));
        assert_eq!(remaining, Rect::new(0, 10, 80, 0));
    }

    #[test]
    fn offset_area_split() {
        let area = Rect::new(5, 10, 40, 20);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fixed(5), Constraint::Fill],
        );
        assert_eq!(rects[0], Rect::new(5, 10, 40, 5));
        assert_eq!(rects[1], Rect::new(5, 15, 40, 15));
    }
}
