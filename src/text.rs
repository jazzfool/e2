use crate::*;
use std::path::Path;
use wgpu_glyph::ab_glyph;

/// Font data that is readily available to be rendered.
///
/// This **requires** a depth stencil state as text draw calls cannot be visually interleaved with regular draw calls otherwise.
#[derive(Debug)]
pub struct FontBrush {
    pub brush: wgpu_glyph::GlyphBrush<wgpu::DepthStencilState>,
}

impl FontBrush {
    /// Loads a font from a path.
    ///
    /// See [FontBrush::new].
    pub fn from_path(
        path: impl AsRef<Path>,
        cx: &Context,
        format: wgpu::TextureFormat,
        depth_stencil: wgpu::DepthStencilState,
    ) -> Result<Self> {
        Self::from_vec(std::fs::read(path)?, cx, format, depth_stencil)
    }

    /// Loads a font from owned font data.
    ///
    /// See [FontBrush::new].
    pub fn from_vec(
        data: Vec<u8>,
        cx: &Context,
        format: wgpu::TextureFormat,
        depth_stencil: wgpu::DepthStencilState,
    ) -> Result<Self> {
        Ok(Self::new(
            ab_glyph::FontArc::try_from_vec(data)?,
            cx,
            format,
            depth_stencil,
        ))
    }

    /// Loads a font from borrowed font data.
    ///
    /// See [FontBrush::new].
    pub fn from_slice(
        data: &'static [u8],
        cx: &Context,
        format: wgpu::TextureFormat,
        depth_stencil: wgpu::DepthStencilState,
    ) -> Result<Self> {
        Ok(Self::new(
            ab_glyph::FontArc::try_from_slice(data)?,
            cx,
            format,
            depth_stencil,
        ))
    }

    /// Creates a new [FontBrush] from an existing [ab_glyph::FontArc].
    ///
    /// `format` and `depth_stencil` state specify information about the color target and depth target
    ///  that the text will be rendered into.
    pub fn new(
        handle: ab_glyph::FontArc,
        cx: &Context,
        format: wgpu::TextureFormat,
        depth_stencil: wgpu::DepthStencilState,
    ) -> Self {
        let brush = wgpu_glyph::GlyphBrushBuilder::using_font(handle)
            .depth_stencil_state(depth_stencil)
            .build(&cx.device, format);
        FontBrush { brush }
    }
}

/// Text rendering helper type which renders text using a [FontBrush].
pub struct TextRenderer {
    staging_belt: wgpu::util::StagingBelt,
}

impl TextRenderer {
    /// Create a new [TextRenderer].
    pub fn new() -> Self {
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        TextRenderer { staging_belt }
    }

    /// Frees the staging belt used when rendering the text.
    ///
    /// Call this *after* [Frame::submit].
    #[inline]
    pub fn free(&mut self) -> impl std::future::Future<Output = ()> + Send {
        self.staging_belt.recall()
    }

    /// Commits all the draw data.
    ///
    /// Call this *before* [Frame::submit] but after all [TextRenderer::draw].
    #[inline]
    pub fn submit(&mut self) {
        self.staging_belt.finish();
    }

    /// Draws text into a specified target, with a given transform and optional clipping rectangle.
    ///
    /// Call this as least times as possible, batching text into arrays of [TextDraw] as much as is practicable.
    pub fn draw(
        &mut self,
        cx: &Context,
        font: &mut FontBrush,
        draws: &[TextDraw],
        frame: &mut Frame,
        target: &wgpu::TextureView,
        depth_stencil: wgpu::RenderPassDepthStencilAttachment,
        transform: glam::Mat4,
        clip: Option<Rect>,
    ) -> Result<()> {
        for draw in draws {
            self.queue(font, draw);
        }

        let transform = transform.to_cols_array();
        if let Some(clip) = clip {
            font.brush
                .draw_queued_with_transform_and_scissoring(
                    &cx.device,
                    &mut self.staging_belt,
                    &mut frame.cmd,
                    target,
                    depth_stencil,
                    transform,
                    wgpu_glyph::Region {
                        x: clip.origin.x as _,
                        y: clip.origin.y as _,
                        width: clip.size.x as _,
                        height: clip.size.y as _,
                    },
                )
                .map_err(|s| Error::TextDraw(s))?;
        } else {
            font.brush
                .draw_queued_with_transform(
                    &cx.device,
                    &mut self.staging_belt,
                    &mut frame.cmd,
                    target,
                    depth_stencil,
                    transform,
                )
                .map_err(|s| Error::TextDraw(s))?;
        }

        Ok(())
    }

    fn queue(&self, font: &mut FontBrush, draw: &TextDraw) {
        font.brush.queue(wgpu_glyph::Section {
            screen_position: (draw.origin.x, draw.origin.y),
            bounds: draw
                .bounds
                .map(|bounds| bounds.into())
                .unwrap_or((f32::INFINITY, f32::INFINITY)),
            text: vec![wgpu_glyph::Text::default()
                .with_text(draw.text)
                .with_scale(draw.scale)
                .with_color([draw.color.r, draw.color.g, draw.color.b, draw.color.a])
                .with_z(draw.depth)],
            ..wgpu_glyph::Section::default()
        });
    }
}

/// Draw data for rendering text.
pub struct TextDraw<'a> {
    /// Top-left of the text.
    pub origin: glam::Vec2,
    /// Depth (z position) of the text.
    pub depth: f32,
    /// Layout bounds of the text.
    ///
    /// `None` means unbounded and thus will render as a single line.
    pub bounds: Option<glam::Vec2>,
    /// The text to be renderer.
    pub text: &'a str,
    /// Pixel scale of the text.
    pub scale: f32,
    /// Color of the text.
    pub color: Color,
}

impl<'a> TextDraw<'a> {
    /// Constructor for [TextDraw] without any layout bounds (single line).
    pub fn unbounded(
        origin: glam::Vec2,
        depth: f32,
        text: &'a str,
        scale: f32,
        color: Color,
    ) -> Self {
        TextDraw {
            origin,
            depth,
            bounds: None,
            text,
            scale,
            color,
        }
    }
}
