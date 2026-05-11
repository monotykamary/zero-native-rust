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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointI {
    pub x: i32,
    pub y: i32,
}

impl PointI {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }

    pub fn translate(self, offset: OffsetI) -> Self {
        Self { x: self.x + offset.dx, y: self.y + offset.dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeI {
    pub width: i32,
    pub height: i32,
}

impl SizeI {
    pub fn new(width: i32, height: i32) -> Self { Self { width, height } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectI {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RectI {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_points(a: PointI, b: PointI) -> Self {
        let x0 = a.x.min(b.x);
        let y0 = a.y.min(b.y);
        Self {
            x: x0, y: y0,
            width: a.x.max(b.x) - x0,
            height: a.y.max(b.y) - y0,
        }
    }

    pub fn min_x(self) -> i32 { self.x }
    pub fn max_x(self) -> i32 { self.x + self.width }
    pub fn min_y(self) -> i32 { self.y }
    pub fn max_y(self) -> i32 { self.y + self.height }

    pub fn is_empty(self) -> bool { self.width <= 0 || self.height <= 0 }
    pub fn has_negative_size(self) -> bool { self.width < 0 || self.height < 0 }

    pub fn normalized(self) -> Self {
        let mut r = self;
        if r.width < 0 { r.x += r.width; r.width = -r.width; }
        if r.height < 0 { r.y += r.height; r.height = -r.height; }
        r
    }

    pub fn contains_point(self, point: PointI) -> bool {
        !self.is_empty()
            && point.x >= self.min_x() && point.x < self.max_x()
            && point.y >= self.min_y() && point.y < self.max_y()
    }

    pub fn contains_rect(self, other: Self) -> bool {
        !other.is_empty()
            && other.min_x() >= self.min_x() && other.max_x() <= self.max_x()
            && other.min_y() >= self.min_y() && other.max_y() <= self.max_y()
    }

    pub fn intersection(a: Self, b: Self) -> Self {
        let x0 = a.min_x().max(b.min_x());
        let y0 = a.min_y().max(b.min_y());
        let x1 = a.max_x().min(b.max_x());
        let y1 = a.max_y().min(b.max_y());
        if x1 <= x0 || y1 <= y0 { return Self { x: x0, y: y0, width: 0, height: 0 }; }
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn intersects(a: Self, b: Self) -> bool { !Self::intersection(a, b).is_empty() }

    pub fn union(a: Self, b: Self) -> Self {
        if a.is_empty() { return b; }
        if b.is_empty() { return a; }
        let x0 = a.min_x().min(b.min_x());
        let y0 = a.min_y().min(b.min_y());
        let x1 = a.max_x().max(b.max_x());
        let y1 = a.max_y().max(b.max_y());
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn top_left(self) -> PointI { PointI::new(self.x, self.y) }
    pub fn bottom_right(self) -> PointI { PointI::new(self.max_x(), self.max_y()) }
    pub fn center(self) -> PointI { PointI::new(self.x + self.width / 2, self.y + self.height / 2) }
    pub fn size(self) -> SizeI { SizeI::new(self.width, self.height) }

    pub fn from_size(size: SizeI) -> Self { Self { width: size.width, height: size.height, x: 0, y: 0 } }

    pub fn translate(self, offset: OffsetI) -> Self {
        Self::new(self.x + offset.dx, self.y + offset.dy, self.width, self.height)
    }

    pub fn scale(self, factor: ScaleI) -> Self {
        Self::new(self.x * factor.x, self.y * factor.y, self.width * factor.x, self.height * factor.y)
    }

    pub fn inflate(self, insets: InsetsI) -> Self {
        Self::new(
            self.x.saturating_sub(insets.left),
            self.y.saturating_sub(insets.top),
            self.width + insets.horizontal(),
            self.height + insets.vertical(),
        )
    }

    pub fn deflate(self, insets: InsetsI) -> Self {
        let move_x = insets.left.min(self.width);
        let move_y = insets.top.min(self.height);
        Self {
            x: self.x + move_x,
            y: self.y + move_y,
            width: shrink_extent(self.width, insets.left, insets.right),
            height: shrink_extent(self.height, insets.top, insets.bottom),
        }
    }
}

fn shrink_extent(value: i32, start: i32, end: i32) -> i32 {
    if start >= value { return 0; }
    let remaining = value - start;
    if end >= remaining { return 0; }
    remaining - end
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InsetsI {
    pub top: i32, pub right: i32, pub bottom: i32, pub left: i32,
}

impl InsetsI {
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self { Self { top, right, bottom, left } }
    pub fn all(v: i32) -> Self { Self { top: v, right: v, bottom: v, left: v } }
    pub fn horizontal(self) -> i32 { self.left + self.right }
    pub fn vertical(self) -> i32 { self.top + self.bottom }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetI { pub dx: i32, pub dy: i32 }
impl OffsetI { pub fn new(dx: i32, dy: i32) -> Self { Self { dx, dy } } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleI { pub x: i32, pub y: i32 }
impl ScaleI { pub fn new(x: i32, y: i32) -> Self { Self { x, y } } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstraintsF {
    pub min_width: f32, pub min_height: f32,
    pub max_width: f32, pub max_height: f32,
}

impl ConstraintsF {
    pub fn new(min: SizeF, max: SizeF) -> Self {
        Self { min_width: min.width, min_height: min.height, max_width: max.width, max_height: max.height }
    }

    pub fn clamp_size(self, size: SizeF) -> SizeF {
        SizeF::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rounding { Truncate, Floor, Ceil, Round }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge { Left, Right, Top, Bottom }

// f64 geometry types

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointD { pub x: f64, pub y: f64 }
impl PointD {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }
    pub fn translate(self, offset: OffsetD) -> Self { Self { x: self.x + offset.dx, y: self.y + offset.dy } }
    pub fn scale(self, factor: ScaleD) -> Self { Self { x: self.x * factor.x, y: self.y * factor.y } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeD { pub width: f64, pub height: f64 }
impl SizeD {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };
    pub fn new(width: f64, height: f64) -> Self { Self { width, height } }
    pub fn is_empty(self) -> bool { self.width <= 0.0 || self.height <= 0.0 }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectD { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
impl RectD {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self { Self { x, y, width, height } }
    pub fn from_size(size: SizeD) -> Self { Self { width: size.width, height: size.height, ..Self::ZERO } }
    pub fn min_x(self) -> f64 { self.x }
    pub fn max_x(self) -> f64 { self.x + self.width }
    pub fn min_y(self) -> f64 { self.y }
    pub fn max_y(self) -> f64 { self.y + self.height }
    pub fn size(self) -> SizeD { SizeD::new(self.width, self.height) }
    pub fn top_left(self) -> PointD { PointD::new(self.x, self.y) }
    pub fn center(self) -> PointD { PointD::new(self.x + self.width / 2.0, self.y + self.height / 2.0) }
    pub fn is_empty(self) -> bool { self.width <= 0.0 || self.height <= 0.0 }
    pub fn has_negative_size(self) -> bool { self.width < 0.0 || self.height < 0.0 }
    pub fn normalized(self) -> Self {
        let mut r = self;
        if r.width < 0.0 { r.x += r.width; r.width = -r.width; }
        if r.height < 0.0 { r.y += r.height; r.height = -r.height; }
        r
    }
    pub fn contains_point(self, point: PointD) -> bool {
        !self.is_empty() && point.x >= self.min_x() && point.x < self.max_x() && point.y >= self.min_y() && point.y < self.max_y()
    }
    pub fn intersection(a: Self, b: Self) -> Self {
        let x0 = a.min_x().max(b.min_x());
        let y0 = a.min_y().max(b.min_y());
        let x1 = a.max_x().min(b.max_x());
        let y1 = a.max_y().min(b.max_y());
        if x1 <= x0 || y1 <= y0 { return Self { x: x0, y: y0, width: 0.0, height: 0.0 }; }
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }
    pub fn intersects(a: Self, b: Self) -> bool { !Self::intersection(a, b).is_empty() }
    pub fn union(a: Self, b: Self) -> Self {
        if a.is_empty() { return b; }
        if b.is_empty() { return a; }
        let x0 = a.min_x().min(b.min_x());
        let y0 = a.min_y().min(b.min_y());
        let x1 = a.max_x().max(b.max_x());
        let y1 = a.max_y().max(b.max_y());
        Self::new(x0, y0, x1 - x0, y1 - y0)
    }
    pub fn translate(self, offset: OffsetD) -> Self { Self::new(self.x + offset.dx, self.y + offset.dy, self.width, self.height) }
    pub fn scale(self, factor: ScaleD) -> Self { Self::new(self.x * factor.x, self.y * factor.y, self.width * factor.x, self.height * factor.y) }
    pub fn inflate(self, insets: InsetsD) -> Self {
        Self::new(self.x - insets.left, self.y - insets.top, self.width + insets.horizontal(), self.height + insets.vertical())
    }
    pub fn deflate(self, insets: InsetsD) -> Self {
        Self::new(self.x + insets.left, self.y + insets.top, (self.width - insets.horizontal()).max(0.0), (self.height - insets.vertical()).max(0.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InsetsD { pub top: f64, pub right: f64, pub bottom: f64, pub left: f64 }
impl InsetsD {
    pub fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self { Self { top, right, bottom, left } }
    pub fn all(v: f64) -> Self { Self { top: v, right: v, bottom: v, left: v } }
    pub fn horizontal(self) -> f64 { self.left + self.right }
    pub fn vertical(self) -> f64 { self.top + self.bottom }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetD { pub dx: f64, pub dy: f64 }
impl OffsetD { pub fn new(dx: f64, dy: f64) -> Self { Self { dx, dy } } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleD { pub x: f64, pub y: f64 }
impl ScaleD {
    pub const IDENTITY: Self = Self { x: 1.0, y: 1.0 };
    pub fn uniform(v: f64) -> Self { Self { x: v, y: v } }
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstraintsD { pub min_width: f64, pub min_height: f64, pub max_width: f64, pub max_height: f64 }
impl ConstraintsD {
    pub fn new(min: SizeD, max: SizeD) -> Self {
        Self { min_width: min.width, min_height: min.height, max_width: max.width, max_height: max.height }
    }
    pub fn unconstrained() -> Self { Self { min_width: 0.0, min_height: 0.0, max_width: f64::INFINITY, max_height: f64::INFINITY } }
    pub fn tight(size: SizeD) -> Self { Self { min_width: size.width, min_height: size.height, max_width: size.width, max_height: size.height } }
    pub fn clamp_size(self, size: SizeD) -> SizeD { SizeD::new(size.width.clamp(self.min_width, self.max_width), size.height.clamp(self.min_height, self.max_height)) }
}

// u32 geometry types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointU { pub x: u32, pub y: u32 }
impl PointU {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub fn new(x: u32, y: u32) -> Self { Self { x, y } }
    pub fn translate(self, offset: OffsetU) -> Self { Self { x: self.x.saturating_add(offset.dx), y: self.y.saturating_add(offset.dy) } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeU { pub width: u32, pub height: u32 }
impl SizeU {
    pub const ZERO: Self = Self { width: 0, height: 0 };
    pub fn new(width: u32, height: u32) -> Self { Self { width, height } }
    pub fn is_empty(self) -> bool { self.width == 0 || self.height == 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectU { pub x: u32, pub y: u32, pub width: u32, pub height: u32 }
impl RectU {
    pub const ZERO: Self = Self { x: 0, y: 0, width: 0, height: 0 };
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self { Self { x, y, width, height } }
    pub fn from_size(size: SizeU) -> Self { Self { width: size.width, height: size.height, x: 0, y: 0 } }
    pub fn min_x(self) -> u32 { self.x }
    pub fn max_x(self) -> u32 { self.x.saturating_add(self.width) }
    pub fn min_y(self) -> u32 { self.y }
    pub fn max_y(self) -> u32 { self.y.saturating_add(self.height) }
    pub fn size(self) -> SizeU { SizeU::new(self.width, self.height) }
    pub fn is_empty(self) -> bool { self.width == 0 || self.height == 0 }
    pub fn contains_point(self, point: PointU) -> bool {
        !self.is_empty() && point.x >= self.min_x() && point.x < self.max_x() && point.y >= self.min_y() && point.y < self.max_y()
    }
    pub fn intersection(a: Self, b: Self) -> Self {
        let x0 = a.min_x().max(b.min_x());
        let y0 = a.min_y().max(b.min_y());
        let x1 = a.max_x().min(b.max_x());
        let y1 = a.max_y().min(b.max_y());
        if x1 <= x0 || y1 <= y0 { return Self { x: x0, y: y0, width: 0, height: 0 }; }
        Self::new(x0, y0, x1 - x0, y1 - y0)
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsetsU { pub top: u32, pub right: u32, pub bottom: u32, pub left: u32 }
impl InsetsU {
    pub fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self { Self { top, right, bottom, left } }
    pub fn all(v: u32) -> Self { Self { top: v, right: v, bottom: v, left: v } }
    pub fn horizontal(self) -> u32 { self.left.saturating_add(self.right) }
    pub fn vertical(self) -> u32 { self.top.saturating_add(self.bottom) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffsetU { pub dx: u32, pub dy: u32 }
impl OffsetU { pub fn new(dx: u32, dy: u32) -> Self { Self { dx, dy } } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleU { pub x: u32, pub y: u32 }
impl ScaleU { pub fn new(x: u32, y: u32) -> Self { Self { x, y } } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstraintsU { pub min_width: u32, pub min_height: u32, pub max_width: u32, pub max_height: u32 }
impl ConstraintsU {
    pub fn new(min: SizeU, max: SizeU) -> Self {
        Self { min_width: min.width, min_height: min.height, max_width: max.width, max_height: max.height }
    }
    pub fn unconstrained() -> Self { Self { min_width: 0, min_height: 0, max_width: u32::MAX, max_height: u32::MAX } }
    pub fn tight(size: SizeU) -> Self { Self { min_width: size.width, min_height: size.height, max_width: size.width, max_height: size.height } }
    pub fn clamp_size(self, size: SizeU) -> SizeU { SizeU::new(size.width.clamp(self.min_width, self.max_width), size.height.clamp(self.min_height, self.max_height)) }
}

// Conversion functions

pub fn convert_f32_to_i32(val: f32, rounding: Rounding) -> i32 {
    match rounding {
        Rounding::Truncate => val as i32,
        Rounding::Floor => val.floor() as i32,
        Rounding::Ceil => val.ceil() as i32,
        Rounding::Round => val.round() as i32,
    }
}

pub fn convert_i32_to_f32(val: i32) -> f32 { val as f32 }

impl RectF {
    pub fn convert_to_i32(self, rounding: Rounding) -> RectI {
        RectI::new(
            convert_f32_to_i32(self.x, rounding),
            convert_f32_to_i32(self.y, rounding),
            convert_f32_to_i32(self.width, rounding),
            convert_f32_to_i32(self.height, rounding),
        )
    }

    pub fn snap_out(self) -> RectI {
        let x0 = self.min_x().floor() as i32;
        let y0 = self.min_y().floor() as i32;
        let x1 = self.max_x().ceil() as i32;
        let y1 = self.max_y().ceil() as i32;
        RectI::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn snap_in(self) -> RectI {
        let x0 = self.min_x().ceil() as i32;
        let y0 = self.min_y().ceil() as i32;
        let x1 = self.max_x().floor() as i32;
        let y1 = self.max_y().floor() as i32;
        if x1 <= x0 || y1 <= y0 { return RectI::new(x0, y0, 0, 0); }
        RectI::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn split(self, edge: Edge, amount: f32) -> [RectF; 2] {
        match edge {
            Edge::Left => {
                let clamped = amount.clamp(0.0, self.width);
                [RectF::new(self.x, self.y, clamped, self.height), RectF::new(self.x + clamped, self.y, self.width - clamped, self.height)]
            }
            Edge::Right => {
                let clamped = amount.clamp(0.0, self.width);
                [RectF::new(self.max_x() - clamped, self.y, clamped, self.height), RectF::new(self.x, self.y, self.width - clamped, self.height)]
            }
            Edge::Top => {
                let clamped = amount.clamp(0.0, self.height);
                [RectF::new(self.x, self.y, self.width, clamped), RectF::new(self.x, self.y + clamped, self.width, self.height - clamped)]
            }
            Edge::Bottom => {
                let clamped = amount.clamp(0.0, self.height);
                [RectF::new(self.x, self.max_y() - clamped, self.width, clamped), RectF::new(self.x, self.y, self.width, self.height - clamped)]
            }
        }
    }

    pub fn split_proportion(self, edge: Edge, proportion: f32) -> [RectF; 2] {
        let clamped = proportion.clamp(0.0, 1.0);
        let extent = match edge { Edge::Left | Edge::Right => self.width, Edge::Top | Edge::Bottom => self.height };
        self.split(edge, extent * clamped)
    }

    pub fn clamp_to_size(self, constraints: ConstraintsF) -> Self {
        let clamped = constraints.clamp_size(self.size());
        Self { x: self.x, y: self.y, width: clamped.width, height: clamped.height }
    }

    pub fn outset(self, insets: InsetsF) -> Self { self.inflate(insets) }
    pub fn inset(self, insets: InsetsF) -> Self { self.deflate(insets) }
}

impl RectI {
    pub fn snap_out_f(self) -> RectF { RectF::new(self.x as f32, self.y as f32, self.width as f32, self.height as f32) }
}

impl SizeF {
    pub fn all(value: f32) -> Self { Self { width: value, height: value } }
    pub fn clamp(self, constraints: ConstraintsF) -> Self { constraints.clamp_size(self) }
}

impl InsetsF {
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
}

impl ConstraintsF {
    pub fn unconstrained() -> Self { Self { min_width: 0.0, min_height: 0.0, max_width: f32::INFINITY, max_height: f32::INFINITY } }
    pub fn tight(size: SizeF) -> Self { Self { min_width: size.width, min_height: size.height, max_width: size.width, max_height: size.height } }
    pub fn loose(max_size: SizeF) -> Self { Self { min_width: 0.0, min_height: 0.0, max_width: max_size.width, max_height: max_size.height } }
}

impl ConstraintsI {
    pub fn new(min: SizeI, max: SizeI) -> Self {
        Self { min_width: min.width, min_height: min.height, max_width: max.width, max_height: max.height }
    }
    pub fn unconstrained() -> Self { Self { min_width: 0, min_height: 0, max_width: i32::MAX, max_height: i32::MAX } }
    pub fn tight(size: SizeI) -> Self { Self { min_width: size.width, min_height: size.height, max_width: size.width, max_height: size.height } }
    pub fn clamp_size(self, size: SizeI) -> SizeI {
        SizeI::new(size.width.clamp(self.min_width, self.max_width), size.height.clamp(self.min_height, self.max_height))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstraintsI { pub min_width: i32, pub min_height: i32, pub max_width: i32, pub max_height: i32 }

pub fn logical_to_physical(rect: RectF, scale_factor: f32) -> RectI {
    rect.scale(ScaleF::uniform(scale_factor)).snap_out()
}

pub fn physical_to_logical(rect: RectI, scale_factor: f32) -> RectF {
    let scaled = RectF::new(rect.x as f32, rect.y as f32, rect.width as f32, rect.height as f32);
    scaled.scale(ScaleF::uniform(1.0 / scale_factor))
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

    // Integer geometry tests matching Zig's rect accessors and constructors
    #[test]
    fn rect_i_accessors_and_constructors() {
        let rect = RectI::new(10, 20, 30, 40);
        assert_eq!(10, rect.min_x());
        assert_eq!(40, rect.max_x());
        assert_eq!(20, rect.min_y());
        assert_eq!(60, rect.max_y());
        assert_eq!(SizeI::new(30, 40), rect.size());
        assert_eq!(PointI::new(10, 20), rect.top_left());
        assert_eq!(PointI::new(40, 60), rect.bottom_right());
        assert_eq!(PointI::new(25, 40), rect.center());
        assert_eq!(RectI::new(0, 0, 30, 40), RectI::from_size(SizeI::new(30, 40)));
        assert_eq!(RectI::new(10, 20, 30, 40), RectI::from_points(PointI::new(40, 60), PointI::new(10, 20)));
    }

    #[test]
    fn rect_i_contains_point_half_open() {
        let r = RectI::new(10, 20, 30, 40);
        assert!(r.contains_point(PointI::new(10, 20)));
        assert!(r.contains_point(PointI::new(39, 59)));
        assert!(!r.contains_point(PointI::new(40, 59)));
        assert!(!r.contains_point(PointI::new(39, 60)));
        assert!(!r.contains_point(PointI::new(9, 20)));
        assert!(!r.contains_point(PointI::new(10, 19)));
    }

    #[test]
    fn rect_i_contains_rect_inclusive_far_edge() {
        let outer = RectI::new(0, 0, 100, 100);
        assert!(outer.contains_rect(RectI::new(0, 0, 100, 100)));
        assert!(outer.contains_rect(RectI::new(10, 10, 20, 20)));
        assert!(!outer.contains_rect(RectI::new(90, 90, 20, 20)));
        assert!(!outer.contains_rect(RectI::new(10, 10, 0, 20)));
    }

    #[test]
    fn rect_i_empty() {
        assert!(RectI::new(0, 0, 0, 10).is_empty());
        assert!(RectI::new(0, 0, 10, 0).is_empty());
        assert!(RectI::new(0, 0, -1, 10).is_empty());
        assert!(!RectI::new(0, 0, 1, 10).is_empty());
    }

    #[test]
    fn rect_i_normalized() {
        assert_eq!(
            RectI::new(15, 30, -10, -20).normalized(),
            RectI::new(5, 10, 10, 20),
        );
    }

    #[test]
    fn rect_i_intersection() {
        let a = RectI::new(0, 0, 100, 100);
        assert_eq!(
            RectI::intersection(a, RectI::new(50, 50, 100, 100)),
            RectI::new(50, 50, 50, 50),
        );
        assert!(RectI::intersection(a, RectI::new(100, 20, 10, 10)).is_empty());
        assert!(RectI::intersects(a, RectI::new(99, 99, 1, 1)));
        assert!(!RectI::intersects(a, RectI::new(100, 99, 1, 1)));
    }

    #[test]
    fn rect_i_union_skips_empty() {
        let a = RectI::new(0, 0, 10, 10);
        let b = RectI::new(5, 20, 10, 10);
        assert_eq!(RectI::union(a, b), RectI::new(0, 0, 15, 30));
        let empty = RectI::new(0, 0, 0, 0);
        assert_eq!(RectI::union(a, empty), a);
        assert_eq!(RectI::union(empty, b), b);
    }

    #[test]
    fn rect_i_inflate_deflate() {
        let rect = RectI::new(10, 10, 100, 50);
        let insets = InsetsI::new(5, 10, 15, 20);
        assert_eq!(30, insets.horizontal());
        assert_eq!(20, insets.vertical());
        assert_eq!(RectI::new(30, 15, 70, 30), rect.deflate(insets));
        assert_eq!(RectI::new(-10, 5, 130, 70), rect.inflate(insets));
    }

    #[test]
    fn rect_i_insets_collapse() {
        let rect = RectI::new(10, 10, 20, 20);
        let collapsed = rect.deflate(InsetsI::all(50));
        assert!(collapsed.is_empty());
    }

    #[test]
    fn rect_i_translate_scale() {
        let point = PointI::new(2, 3);
        let rect = RectI::new(1, 2, 3, 4);
        assert_eq!(PointI::new(7, 1), point.translate(OffsetI::new(5, -2)));
        assert_eq!(RectI::new(6, 0, 3, 4), rect.translate(OffsetI::new(5, -2)));
        assert_eq!(RectI::new(2, 6, 6, 12), rect.scale(ScaleI::new(2, 3)));
    }

    #[test]
    fn constraints_clamp_sizes() {
        let c = ConstraintsF::new(SizeF::new(10.0, 20.0), SizeF::new(100.0, 200.0));
        assert_eq!(SizeF::new(10.0, 20.0), c.clamp_size(SizeF::new(5.0, 10.0)));
        assert_eq!(SizeF::new(50.0, 60.0), c.clamp_size(SizeF::new(50.0, 60.0)));
        assert_eq!(SizeF::new(100.0, 200.0), c.clamp_size(SizeF::new(150.0, 300.0)));
    }

    #[test]
    fn f64_types_zero_and_constructors() {
        assert_eq!(PointD::ZERO, PointD::new(0.0, 0.0));
        assert_eq!(SizeD::ZERO, SizeD::new(0.0, 0.0));
        assert_eq!(RectD::ZERO, RectD::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(ScaleD::IDENTITY, ScaleD::new(1.0, 1.0));
    }

    #[test]
    fn u32_types_zero_and_constructors() {
        assert_eq!(PointU::ZERO, PointU::new(0, 0));
        assert_eq!(SizeU::ZERO, SizeU::new(0, 0));
        assert_eq!(RectU::ZERO, RectU::new(0, 0, 0, 0));
    }

    #[test]
    fn rect_d_accessors() {
        let r = RectD::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(10.0, r.min_x());
        assert_eq!(40.0, r.max_x());
        assert_eq!(20.0, r.min_y());
        assert_eq!(60.0, r.max_y());
        assert_eq!(PointD::new(25.0, 40.0), r.center());
    }

    #[test]
    fn rect_u_accessors() {
        let r = RectU::new(10, 20, 30, 40);
        assert_eq!(10, r.min_x());
        assert_eq!(40, r.max_x());
        assert!(r.contains_point(PointU::new(15, 25)));
        assert!(!r.contains_point(PointU::new(40, 25)));
    }

    #[test]
    fn rounding_conversion() {
        let val = 1.5f32;
        assert_eq!(1, convert_f32_to_i32(val, Rounding::Truncate));
        assert_eq!(1, convert_f32_to_i32(val, Rounding::Floor));
        assert_eq!(2, convert_f32_to_i32(val, Rounding::Ceil));
        assert_eq!(2, convert_f32_to_i32(val, Rounding::Round));
    }

    #[test]
    fn rect_f_convert_to_i32() {
        let r = RectF::new(1.25, 2.75, 10.5, 20.25);
        assert_eq!(RectI::new(1, 2, 10, 20), r.convert_to_i32(Rounding::Floor));
        assert_eq!(RectI::new(2, 3, 11, 21), r.convert_to_i32(Rounding::Ceil));
    }

    #[test]
    fn snap_out_and_snap_in() {
        let r = RectF::new(1.25, 2.75, 10.5, 20.25);
        assert_eq!(RectI::new(1, 2, 11, 21), r.snap_out());
        assert_eq!(RectI::new(2, 3, 9, 20), r.snap_in());
    }

    #[test]
    fn split_by_edge() {
        let r = RectF::new(0.0, 0.0, 100.0, 50.0);
        let [left, right] = r.split(Edge::Left, 25.0);
        assert_eq!(RectF::new(0.0, 0.0, 25.0, 50.0), left);
        assert_eq!(RectF::new(25.0, 0.0, 75.0, 50.0), right);
        let [top, bottom] = r.split(Edge::Top, 10.0);
        assert_eq!(RectF::new(0.0, 0.0, 100.0, 10.0), top);
        assert_eq!(RectF::new(0.0, 10.0, 100.0, 40.0), bottom);
    }

    #[test]
    fn split_proportion() {
        let r = RectF::new(0.0, 0.0, 100.0, 50.0);
        let [left, _] = r.split_proportion(Edge::Left, 0.25);
        assert_eq!(RectF::new(0.0, 0.0, 25.0, 50.0), left);
    }

    #[test]
    fn logical_physical_conversion() {
        let logical = RectF::new(0.25, 1.25, 10.25, 20.25);
        let physical = logical_to_physical(logical, 2.0);
        assert_eq!(RectI::new(0, 2, 21, 41), physical);
        let back = physical_to_logical(physical, 2.0);
        assert_eq!(0.0, back.x);
        assert_eq!(1.0, back.y);
        assert!((back.width - 10.5).abs() < 0.01);
    }

    #[test]
    fn constraints_i_and_u() {
        let ci = ConstraintsI::new(SizeI::new(10, 20), SizeI::new(100, 200));
        assert_eq!(SizeI::new(10, 20), ci.clamp_size(SizeI::new(5, 10)));
        let cu = ConstraintsU::tight(SizeU::new(42, 64));
        assert_eq!(SizeU::new(42, 64), cu.clamp_size(SizeU::new(1, 2)));
    }
}

