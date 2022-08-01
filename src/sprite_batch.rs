use crate::*;
use std::borrow::Cow;

/// This is a version of [BatchRenderer] that is designed for rendering batches of sprites (textured or colored rectangle).
#[derive(Debug)]
pub struct SpriteBatchRenderer {
    renderer: BatchRenderer,
    rect: Mesh,
    white: Texture,
    matrix: glam::Mat4,
}

impl SpriteBatchRenderer {
    /// Creates a new [SpriteBatchRenderer].
    ///
    /// See [BatchRenderer::new].
    pub fn new(cx: &Context, pipeline: &BatchRenderPipeline) -> Self {
        let renderer = BatchRenderer::new(pipeline);

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

        SpriteBatchRenderer {
            renderer,
            rect,
            white,
            matrix: glam::Mat4::IDENTITY,
        }
    }

    /// Resets the previously allocated buffers, making them available for reuse.
    ///
    /// Call this at the start or end of every frame in order to maintain acceptable spatial performance.
    pub fn reset(&mut self) {
        self.renderer.reset();
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

    /// Draws a batch of sprites, either textured or colored.
    ///
    /// See [BatchRenderer::draw].
    pub fn draw<'a>(
        &mut self,
        cx: &Context,
        pass: &mut ArenaRenderPass,
        batch: impl Into<SpriteBatch<'a>>,
    ) {
        let (texture, draws) = match batch.into() {
            SpriteBatch::Textured { texture, draws } => (
                texture,
                draws
                    .iter()
                    .map(|draw| BatchDraw {
                        color: draw.color,
                        src_rect: draw.src_rect,
                        transform: self.matrix * rect_matrix(draw.rect, draw.rotation),
                    })
                    .collect::<Vec<_>>(),
            ),
            SpriteBatch::Color { draws } => (
                &self.white,
                draws
                    .iter()
                    .map(|draw| BatchDraw {
                        color: draw.color,
                        src_rect: Rect::ONE,
                        transform: self.matrix * rect_matrix(draw.rect, draw.rotation),
                    })
                    .collect(),
            ),
        };

        self.renderer.draw(cx, pass, &self.rect, texture, &draws);
    }
}

impl Slot3BatchRenderer for SpriteBatchRenderer {
    #[inline]
    fn bind(&mut self, storage: u32, texture: u32, sampler: u32) {
        self.renderer.bind(storage, texture, sampler);
    }
}

/// Draw data for a single instance in a textured batched sprite draw.
pub struct SpriteBatchTexturedDraw {
    /// Color to multiply texture color with.
    /// Using [Color::WHITE] will mean the texture will render as-is.
    pub color: Color,
    /// UV sub-rectangle to use.
    /// Using [Rect::ONE] will mean that the full UV space is available.
    pub src_rect: Rect,
    /// Where to draw the sprite.
    pub rect: Rect,
    /// Rotation (in radians) of the sprite.
    pub rotation: f32,
}

/// Draw data for a single instance in a non-textured batched sprite draw.
pub struct SpriteBatchColorDraw {
    /// Color to fill with.
    pub color: Color,
    /// Where to draw the sprite.
    pub rect: Rect,
    /// Rotation (in radians) of the sprite.
    pub rotation: f32,
}

/// Sprite batch data, either in the form of texture sprites or colored sprites.
pub enum SpriteBatch<'a> {
    Textured {
        texture: &'a Texture,
        draws: &'a [SpriteBatchTexturedDraw],
    },
    Color {
        draws: &'a [SpriteBatchColorDraw],
    },
}

impl<'a> From<(&'a Texture, &'a [SpriteBatchTexturedDraw])> for SpriteBatch<'a> {
    #[inline]
    fn from((texture, draws): (&'a Texture, &'a [SpriteBatchTexturedDraw])) -> Self {
        SpriteBatch::Textured { texture, draws }
    }
}

impl<'a> From<&'a [SpriteBatchColorDraw]> for SpriteBatch<'a> {
    #[inline]
    fn from(draws: &'a [SpriteBatchColorDraw]) -> Self {
        SpriteBatch::Color { draws }
    }
}
