#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}

impl PointF {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn translate(self, offset: OffsetF) -> Self {
        Self {
            x: self.x + offset.dx,
            y: self.y + offset.dy,
        }
    }

    pub fn scale(self, factor: ScaleF) -> Self {
        Self {
            x: self.x * factor.x,
            y: self.y * factor.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeF {
    pub width: f32,
    pub height: f32,
}

impl SizeF {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectF {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_size(size: SizeF) -> Self {
        Self {
            width: size.width,
            height: size.height,
            ..Self::ZERO
        }
    }

    pub fn min_x(self) -> f32 {
        self.x
    }

    pub fn max_x(self) -> f32 {
        self.x + self.width
    }

    pub fn min_y(self) -> f32 {
        self.y
    }

    pub fn max_y(self) -> f32 {
        self.y + self.height
    }

    pub fn size(self) -> SizeF {
        SizeF::new(self.width, self.height)
    }

    pub fn center(self) -> PointF {
        PointF::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn has_negative_size(self) -> bool {
        self.width < 0.0 || self.height < 0.0
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn intersection(a: Self, b: Self) -> Self {
        let x0 = a.min_x().max(b.min_x());
        let y0 = a.min_y().max(b.min_y());
        let x1 = a.max_x().min(b.max_x());
        let y1 = a.max_y().min(b.max_y());
        if x1 <= x0 || y1 <= y0 {
            return Self {
                x: x0,
                y: y0,
                ..Self::ZERO
            };
        }
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn intersects(a: Self, b: Self) -> bool {
        !Self::intersection(a, b).is_empty()
    }

    pub fn translate(self, offset: OffsetF) -> Self {
        Self::new(
            self.x + offset.dx,
            self.y + offset.dy,
            self.width,
            self.height,
        )
    }

    pub fn scale(self, factor: ScaleF) -> Self {
        Self::new(
            self.x * factor.x,
            self.y * factor.y,
            self.width * factor.x,
            self.height * factor.y,
        )
    }

    pub fn contains_point(self, point: PointF) -> bool {
        !self.is_empty()
            && point.x >= self.min_x()
            && point.x < self.max_x()
            && point.y >= self.min_y()
            && point.y < self.max_y()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetF {
    pub dx: f32,
    pub dy: f32,
}

impl OffsetF {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleF {
    pub x: f32,
    pub y: f32,
}

impl ScaleF {
    pub fn uniform(v: f32) -> Self {
        Self { x: v, y: v }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InsetsF {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl InsetsF {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_accessors() {
        let r = RectF::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.min_x(), 10.0);
        assert_eq!(r.max_x(), 40.0);
        assert_eq!(r.min_y(), 20.0);
        assert_eq!(r.max_y(), 60.0);
        assert_eq!(r.size(), SizeF::new(30.0, 40.0));
    }

    #[test]
    fn rect_contains_point_half_open() {
        let r = RectF::new(10.0, 20.0, 30.0, 40.0);
        assert!(r.contains_point(PointF::new(10.0, 20.0)));
        assert!(r.contains_point(PointF::new(39.0, 59.0)));
        assert!(!r.contains_point(PointF::new(40.0, 59.0)));
        assert!(!r.contains_point(PointF::new(39.0, 60.0)));
    }

    #[test]
    fn rect_intersection() {
        let a = RectF::new(0.0, 0.0, 100.0, 100.0);
        let b = RectF::new(50.0, 50.0, 100.0, 100.0);
        assert_eq!(RectF::intersection(a, b), RectF::new(50.0, 50.0, 50.0, 50.0));
        assert!(RectF::intersects(a, b));
    }

    #[test]
    fn rect_empty() {
        assert!(RectF::new(0.0, 0.0, 0.0, 10.0).is_empty());
        assert!(RectF::new(0.0, 0.0, -1.0, 10.0).is_empty());
        assert!(!RectF::new(0.0, 0.0, 1.0, 10.0).is_empty());
    }
}
