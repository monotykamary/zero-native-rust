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
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };

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
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_size(size: SizeF) -> Self {
        Self { width: size.width, height: size.height, ..Self::ZERO }
    }

    pub fn min_x(self) -> f32 { self.x }
    pub fn max_x(self) -> f32 { self.x + self.width }
    pub fn min_y(self) -> f32 { self.y }
    pub fn max_y(self) -> f32 { self.y + self.height }

    pub fn size(self) -> SizeF {
        SizeF::new(self.width, self.height)
    }

    pub fn top_left(self) -> PointF {
        PointF::new(self.x, self.y)
    }

    pub fn bottom_right(self) -> PointF {
        PointF::new(self.max_x(), self.max_y())
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

    pub fn normalized(self) -> Self {
        let mut r = self;
        if r.width < 0.0 {
            r.x += r.width;
            r.width = -r.width;
        }
        if r.height < 0.0 {
            r.y += r.height;
            r.height = -r.height;
        }
        r
    }

    pub fn contains_point(self, point: PointF) -> bool {
        !self.is_empty()
            && point.x >= self.min_x()
            && point.x < self.max_x()
            && point.y >= self.min_y()
            && point.y < self.max_y()
    }

    pub fn contains_rect(self, other: Self) -> bool {
        !other.is_empty()
            && other.min_x() >= self.min_x()
            && other.max_x() <= self.max_x()
            && other.min_y() >= self.min_y()
            && other.max_y() <= self.max_y()
    }

    pub fn intersection(a: Self, b: Self) -> Self {
        let x0 = a.min_x().max(b.min_x());
        let y0 = a.min_y().max(b.min_y());
        let x1 = a.max_x().min(b.max_x());
        let y1 = a.max_y().min(b.max_y());
        if x1 <= x0 || y1 <= y0 {
            return Self { x: x0, y: y0, ..Self::ZERO };
        }
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn intersects(a: Self, b: Self) -> bool {
        !Self::intersection(a, b).is_empty()
    }

    pub fn union(a: Self, b: Self) -> Self {
        if a.is_empty() { return b; }
        if b.is_empty() { return a; }
        let x0 = a.min_x().min(b.min_x());
        let y0 = a.min_y().min(b.min_y());
        let x1 = a.max_x().max(b.max_x());
        let y1 = a.max_y().max(b.max_y());
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn inflate(self, insets: InsetsF) -> Self {
        Self::new(
            self.x - insets.left,
            self.y - insets.top,
            self.width + insets.horizontal(),
            self.height + insets.vertical(),
        )
    }

    pub fn deflate(self, insets: InsetsF) -> Self {
        Self::new(
            self.x + insets.left,
            self.y + insets.top,
            self.width - insets.horizontal(),
            self.height - insets.vertical(),
        )
    }

    pub fn translate(self, offset: OffsetF) -> Self {
        Self::new(self.x + offset.dx, self.y + offset.dy, self.width, self.height)
    }

    pub fn scale(self, factor: ScaleF) -> Self {
        Self::new(self.x * factor.x, self.y * factor.y, self.width * factor.x, self.height * factor.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetF {
    pub dx: f32,
    pub dy: f32,
}

impl OffsetF {
    pub fn new(dx: f32, dy: f32) -> Self { Self { dx, dy } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleF {
    pub x: f32,
    pub y: f32,
}

impl ScaleF {
    pub const IDENTITY: Self = Self { x: 1.0, y: 1.0 };

    pub fn uniform(v: f32) -> Self { Self { x: v, y: v } }

    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
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
        Self { top, right, bottom, left }
    }

    pub fn all(v: f32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }

    pub fn horizontal(self) -> f32 { self.left + self.right }
    pub fn vertical(self) -> f32 { self.top + self.bottom }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_aliases() {
        assert_eq!(PointF::ZERO, PointF::new(0.0, 0.0));
        assert_eq!(SizeF::ZERO, SizeF::new(0.0, 0.0));
        assert_eq!(RectF::ZERO, RectF::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(ScaleF::IDENTITY, ScaleF::new(1.0, 1.0));
    }

    #[test]
    fn rect_accessors() {
        let r = RectF::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.min_x(), 10.0);
        assert_eq!(r.max_x(), 40.0);
        assert_eq!(r.min_y(), 20.0);
        assert_eq!(r.max_y(), 60.0);
        assert_eq!(r.size(), SizeF::new(30.0, 40.0));
        assert_eq!(r.top_left(), PointF::new(10.0, 20.0));
        assert_eq!(r.bottom_right(), PointF::new(40.0, 60.0));
        assert_eq!(r.center(), PointF::new(25.0, 40.0));
        assert_eq!(RectF::from_size(SizeF::new(30.0, 40.0)), RectF::new(0.0, 0.0, 30.0, 40.0));
    }

    #[test]
    fn rect_contains_point_half_open() {
        let r = RectF::new(10.0, 20.0, 30.0, 40.0);
        assert!(r.contains_point(PointF::new(10.0, 20.0)));
        assert!(r.contains_point(PointF::new(39.0, 59.0)));
        assert!(!r.contains_point(PointF::new(40.0, 59.0)));
        assert!(!r.contains_point(PointF::new(39.0, 60.0)));
        assert!(!r.contains_point(PointF::new(9.0, 20.0)));
        assert!(!r.contains_point(PointF::new(10.0, 19.0)));
    }

    #[test]
    fn rect_contains_rect_inclusive_far_edge() {
        let outer = RectF::new(0.0, 0.0, 100.0, 100.0);
        assert!(outer.contains_rect(RectF::new(0.0, 0.0, 100.0, 100.0)));
        assert!(outer.contains_rect(RectF::new(10.0, 10.0, 20.0, 20.0)));
        assert!(!outer.contains_rect(RectF::new(90.0, 90.0, 20.0, 20.0)));
        assert!(!outer.contains_rect(RectF::new(10.0, 10.0, 0.0, 20.0)));
    }

    #[test]
    fn rect_empty() {
        assert!(RectF::new(0.0, 0.0, 0.0, 10.0).is_empty());
        assert!(RectF::new(0.0, 0.0, 10.0, 0.0).is_empty());
        assert!(RectF::new(0.0, 0.0, -1.0, 10.0).is_empty());
        assert!(!RectF::new(0.0, 0.0, 1.0, 10.0).is_empty());
    }

    #[test]
    fn rect_normalized() {
        assert_eq!(
            RectF::new(15.0, 30.0, -10.0, -20.0).normalized(),
            RectF::new(5.0, 10.0, 10.0, 20.0),
        );
    }

    #[test]
    fn rect_intersection() {
        let a = RectF::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(
            RectF::intersection(a, RectF::new(50.0, 50.0, 100.0, 100.0)),
            RectF::new(50.0, 50.0, 50.0, 50.0),
        );
        // touching at edge
        assert!(RectF::intersection(a, RectF::new(100.0, 20.0, 10.0, 10.0)).is_empty());
        // contained
        assert_eq!(
            RectF::intersection(a, RectF::new(10.0, 10.0, 20.0, 20.0)),
            RectF::new(10.0, 10.0, 20.0, 20.0),
        );
        // no overlap
        assert!(RectF::intersection(a, RectF::new(200.0, 200.0, 10.0, 10.0)).is_empty());
        // intersects
        assert!(RectF::intersects(a, RectF::new(99.0, 99.0, 1.0, 1.0)));
        assert!(!RectF::intersects(a, RectF::new(100.0, 99.0, 1.0, 1.0)));
    }

    #[test]
    fn rect_union_skips_empty() {
        let a = RectF::new(0.0, 0.0, 10.0, 10.0);
        let b = RectF::new(5.0, 20.0, 10.0, 10.0);
        assert_eq!(RectF::union(a, b), RectF::new(0.0, 0.0, 15.0, 30.0));
        assert_eq!(RectF::union(a, RectF::ZERO), a);
        assert_eq!(RectF::union(RectF::ZERO, b), b);
        assert_eq!(RectF::union(RectF::ZERO, RectF::ZERO), RectF::ZERO);
    }

    #[test]
    fn insets_inflate_deflate() {
        let rect = RectF::new(10.0, 10.0, 100.0, 50.0);
        let insets = InsetsF::new(5.0, 10.0, 15.0, 20.0);
        assert_eq!(insets.horizontal(), 30.0);
        assert_eq!(insets.vertical(), 20.0);
        assert_eq!(rect.deflate(insets), RectF::new(30.0, 15.0, 70.0, 30.0));
        assert_eq!(rect.inflate(insets), RectF::new(-10.0, 5.0, 130.0, 70.0));
    }

    #[test]
    fn insets_collapse_rect() {
        let rect = RectF::new(10.0, 10.0, 20.0, 20.0);
        let collapsed = rect.deflate(InsetsF::all(50.0));
        assert_eq!(collapsed.x, 60.0);
        assert_eq!(collapsed.y, 60.0);
        assert!(collapsed.is_empty());
    }

    #[test]
    fn translate_scale_point() {
        let point = PointF::new(2.0, 3.0);
        let rect = RectF::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(point.translate(OffsetF::new(5.0, -2.0)), PointF::new(7.0, 1.0));
        assert_eq!(point.scale(ScaleF::new(2.0, 3.0)), PointF::new(4.0, 9.0));
        assert_eq!(rect.translate(OffsetF::new(5.0, -2.0)), RectF::new(6.0, 0.0, 3.0, 4.0));
        assert_eq!(rect.scale(ScaleF::new(2.0, 3.0)), RectF::new(2.0, 6.0, 6.0, 12.0));
    }
}
