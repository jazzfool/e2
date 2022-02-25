use crate::*;

/// Simple sRGB color type with an alpha channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Color::new(1., 1., 1., 1.);
    pub const BLACK: Self = Color::new(0., 0., 0., 1.);
    pub const RED: Self = Color::new(1., 0., 0., 1.);
    pub const GREEN: Self = Color::new(0., 1., 0., 1.);
    pub const BLUE: Self = Color::new(0., 0., 1., 1.);

    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        wgpu::Color {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        }
    }
}
