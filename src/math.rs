#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: glam::Vec2,
    pub size: glam::Vec2,
}

impl Rect {
    pub const ONE: Self = Rect {
        origin: glam::Vec2::ZERO,
        size: glam::Vec2::ONE,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            origin: glam::Vec2::new(x, y),
            size: glam::Vec2::new(width, height),
        }
    }

    pub fn deflate(&self, x: f32, y: f32) -> Rect {
        Rect::new(
            self.origin.x + x,
            self.origin.y + y,
            self.size.x - x * 2.,
            self.size.y - y * 2.,
        )
    }

    pub fn inflate(&self, x: f32, y: f32) -> Rect {
        self.deflate(-x, -y)
    }
}
