use crate::*;

/// 2D rectangle type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Origin, usually top-left, of the rectangle.
    pub origin: glam::Vec2,
    /// Size, usually positive, of the rectangle.
    pub size: glam::Vec2,
}

impl Rect {
    /// Rectangle at (0, 0) with size of (1, 1).
    pub const ONE: Self = Rect {
        origin: glam::Vec2::ZERO,
        size: glam::Vec2::ONE,
    };

    /// Creates a new rectangle positioned at `(x, y)` with size of `(width, height)`.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            origin: glam::Vec2::new(x, y),
            size: glam::Vec2::new(width, height),
        }
    }

    /// Deflates the sides of the rectangle.
    ///
    /// NB. the width and height will be reduced by `2x` and `2y`, respectively, in total.
    pub fn deflate(&self, x: f32, y: f32) -> Rect {
        Rect::new(
            self.origin.x + x,
            self.origin.y + y,
            self.size.x - x * 2.,
            self.size.y - y * 2.,
        )
    }

    /// [Rect::deflate], but in the opposite direction.
    #[inline]
    pub fn inflate(&self, x: f32, y: f32) -> Rect {
        self.deflate(-x, -y)
    }
}
