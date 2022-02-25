use crate::*;
use std::borrow::Cow;

/// This is a version of [MeshRenderer] that is designed for rendering sprites (textured or colored rectangles).
#[derive(Debug)]
pub struct SpriteRenderer {
    renderer: MeshRenderer,
    rect: Mesh,
    white: Texture,
    matrix: glam::Mat4,
}

impl SpriteRenderer {
    /// Creates a new [SpriteRenderer].
    ///
    /// See [MeshRenderer::new].
    pub fn new(cx: &Context, pipeline: &MeshRenderPipeline) -> Self {
        let renderer = MeshRenderer::new(cx, pipeline);

        let rect = Mesh::new(
            &cx,
            &[
                Vertex {
                    pos: [0., 0.],
                    uv: [0., 0.],
                },
                Vertex {
                    pos: [1., 0.],
                    uv: [1., 0.],
                },
                Vertex {
                    pos: [0., 1.],
                    uv: [0., 1.],
                },
                Vertex {
                    pos: [1., 1.],
                    uv: [1., 1.],
                },
            ],
            &[0, 2, 1, 2, 3, 1],
        );

        let white = ImageTexture {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            pixels: Cow::Borrowed(&[255, 255, 255, 255]),
            width: 1,
            height: 1,
        }
        .create(&cx);

        SpriteRenderer {
            renderer,
            rect,
            white,
            matrix: glam::Mat4::IDENTITY,
        }
    }

    /// Resets the previously allocated buffers, making them available for reuse.
    ///
    /// Call this at the start or end of every frame in order to maintain acceptable spatial performance.
    pub fn free(&mut self) {
        self.renderer.free();
    }

    /// Binds a sampler for use with the proceeding draw calls.
    pub fn bind_sampler<'a>(
        &mut self,
        cx: &Context,
        pass: &'a mut wgpu::RenderPass,
        sampler: &Sampler,
    ) {
        self.renderer.bind_sampler(cx, pass, sampler);
    }

    /// Sets the matrix that is premultiplied against the sprite transformation matrices.
    pub fn set_matrix(&mut self, matrix: glam::Mat4) {
        self.matrix = matrix;
    }

    /// Draws a sprite at `rect` with a given `content` and `rotation`.
    ///
    /// Rotation is in radians.
    pub fn draw<'a>(
        &mut self,
        cx: &Context,
        pass: &mut ArenaRenderPass,
        content: impl Into<SpriteContent<'a>>,
        rect: Rect,
        rotation: f32,
    ) {
        let (texture, src_rect, color) = match content.into() {
            SpriteContent::Textured { texture, src_rect } => (texture, src_rect, Color::WHITE),
            SpriteContent::Color(color) => (&self.white, Rect::ONE, color),
        };

        self.renderer.draw(
            cx,
            pass,
            MeshDraw {
                mesh: &self.rect,
                texture,
                color,
                src_rect,
                transform: glam::Mat4::from_scale_rotation_translation(
                    glam::vec3(rect.size.x, rect.size.y, 1.),
                    glam::Quat::from_rotation_z(rotation),
                    glam::vec3(rect.origin.x, rect.origin.y, 0.),
                ),
            },
        );
    }
}

impl Slot3MeshRenderer for SpriteRenderer {
    #[inline]
    fn bind(&mut self, uniform: u32, texture: u32, sampler: u32) {
        self.renderer.bind(uniform, texture, sampler);
    }
}

/// The visual contents (texture or color) of a sprite.
pub enum SpriteContent<'a> {
    /// The sprite is textured.
    Textured {
        /// Texture to sample.
        texture: &'a Texture,
        /// UV sub-rectangle to use.
        /// Using [Rect::ONE] will mean that the full UV space is available.
        src_rect: Rect,
    },
    /// The sprite has a solid color.
    Color(Color),
}

impl<'a> From<&'a Texture> for SpriteContent<'a> {
    fn from(texture: &'a Texture) -> Self {
        SpriteContent::Textured {
            texture,
            src_rect: Rect::ONE,
        }
    }
}

impl<'a> From<(&'a Texture, Rect)> for SpriteContent<'a> {
    fn from((texture, src_rect): (&'a Texture, Rect)) -> Self {
        SpriteContent::Textured { texture, src_rect }
    }
}

impl<'a> From<Color> for SpriteContent<'a> {
    fn from(color: Color) -> Self {
        SpriteContent::Color(color)
    }
}
