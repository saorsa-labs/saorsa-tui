//! Geometry primitives: Position, Size, Rect.

/// A position in terminal coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Position {
    /// X coordinate (column).
    pub x: u16,
    /// Y coordinate (row).
    pub y: u16,
}

impl Position {
    /// Create a new position.
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl From<(u16, u16)> for Position {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
    }
}

/// A size in terminal cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Size {
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl Size {
    /// Create a new size.
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Returns the area (width * height).
    pub const fn area(self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Returns true if either dimension is zero.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Self { width, height }
    }
}

/// A rectangle defined by position and size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    /// Top-left position.
    pub position: Position,
    /// Dimensions.
    pub size: Size,
}

impl Rect {
    /// Create a new rectangle.
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            position: Position::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// X coordinate of the right edge (exclusive).
    pub const fn right(self) -> u16 {
        self.position.x.saturating_add(self.size.width)
    }

    /// Y coordinate of the bottom edge (exclusive).
    pub const fn bottom(self) -> u16 {
        self.position.y.saturating_add(self.size.height)
    }

    /// Area of the rectangle.
    pub const fn area(self) -> u32 {
        self.size.area()
    }

    /// Returns true if the rectangle has zero area.
    pub const fn is_empty(self) -> bool {
        self.size.is_empty()
    }

    /// Returns true if the point is inside this rectangle.
    pub const fn contains(self, pos: Position) -> bool {
        pos.x >= self.position.x
            && pos.x < self.right()
            && pos.y >= self.position.y
            && pos.y < self.bottom()
    }

    /// Returns true if two rectangles overlap.
    pub const fn intersects(self, other: &Rect) -> bool {
        self.position.x < other.right()
            && self.right() > other.position.x
            && self.position.y < other.bottom()
            && self.bottom() > other.position.y
    }

    /// Returns the intersection of two rectangles, or `None` if they don't overlap.
    pub fn intersection(self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        let x = self.position.x.max(other.position.x);
        let y = self.position.y.max(other.position.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Some(Rect::new(x, y, right - x, bottom - y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_from_tuple() {
        let pos: Position = (5, 10).into();
        assert_eq!(pos, Position::new(5, 10));
    }

    #[test]
    fn size_from_tuple() {
        let sz: Size = (80, 24).into();
        assert_eq!(sz, Size::new(80, 24));
    }

    #[test]
    fn size_area() {
        assert_eq!(Size::new(10, 5).area(), 50);
    }

    #[test]
    fn size_empty() {
        assert!(Size::new(0, 10).is_empty());
        assert!(Size::new(10, 0).is_empty());
        assert!(!Size::new(1, 1).is_empty());
    }

    #[test]
    fn rect_right_bottom() {
        let r = Rect::new(5, 10, 20, 15);
        assert_eq!(r.right(), 25);
        assert_eq!(r.bottom(), 25);
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(10, 10, 20, 20);
        assert!(r.contains(Position::new(10, 10)));
        assert!(r.contains(Position::new(29, 29)));
        assert!(!r.contains(Position::new(30, 30)));
        assert!(!r.contains(Position::new(9, 10)));
    }

    #[test]
    fn rect_intersects() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let c = Rect::new(20, 20, 5, 5);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn rect_intersection() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let i = a.intersection(&b);
        assert_eq!(i, Some(Rect::new(5, 5, 5, 5)));
    }

    #[test]
    fn rect_no_intersection() {
        let a = Rect::new(0, 0, 5, 5);
        let b = Rect::new(10, 10, 5, 5);
        assert_eq!(a.intersection(&b), None);
    }

    #[test]
    fn rect_empty() {
        assert!(Rect::new(0, 0, 0, 5).is_empty());
        assert!(!Rect::new(0, 0, 1, 1).is_empty());
    }

    #[test]
    fn rect_area() {
        assert_eq!(Rect::new(0, 0, 10, 5).area(), 50);
    }

    #[test]
    fn rect_saturating_overflow() {
        let r = Rect::new(u16::MAX, u16::MAX, 10, 10);
        assert_eq!(r.right(), u16::MAX);
        assert_eq!(r.bottom(), u16::MAX);
    }
}
