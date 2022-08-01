use crate::*;
use std::sync::Arc;

/// Simplified render pass descriptor.
#[derive(Debug, Clone)]
pub struct SimpleRenderPass<'a> {
    /// Texture to render to.
    pub target: &'a wgpu::TextureView,
    /// Texture to resolve samples from `target` into.
    /// Only valid when `target.samples > 1`.
    pub resolve: Option<&'a wgpu::TextureView>,
    /// Clear color.
    /// If `None` then the texture is not cleared.
    pub clear: Option<Color>,
    /// Depth texture.
    pub depth_stencil: Option<wgpu::RenderPassDepthStencilAttachment<'a>>,
}

impl<'a> SimpleRenderPass<'a> {
    /// Begins a new [ArenaRenderPass] from the stored pass configuration.
    pub fn begin(self, frame: &'a mut Frame) -> ArenaRenderPass<'a> {
        let pass = frame.cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.target,
                resolve_target: self.resolve,
                ops: wgpu::Operations {
                    load: match self.clear {
                        Some(color) => wgpu::LoadOp::Clear(color.into()),
                        None => wgpu::LoadOp::Load,
                    },
                    store: true,
                },
            })],
            depth_stencil_attachment: self.depth_stencil,
        });

        ArenaRenderPass {
            arena: &frame.arena,
            pass,
        }
    }
}

/// [wgpu::RenderPipeline] equivalent, but with more sensible lifetimes.
///
/// Overrides methods that take [std::sync::Arc] GPU resources and allocate thems on `arena`.
pub struct ArenaRenderPass<'a> {
    pub arena: &'a FrameArena,
    pub pass: wgpu::RenderPass<'a>,
}

impl<'a> ArenaRenderPass<'a> {
    /// See [wgpu::RenderPass::set_pipeline].
    pub fn set_pipeline(&mut self, pipeline: Arc<wgpu::RenderPipeline>) {
        let pipeline = self.arena.render_pipelines.alloc(pipeline);
        self.pass.set_pipeline(pipeline);
    }

    /// See [wgpu::RenderPass::set_bind_group].
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: Arc<wgpu::BindGroup>,
        offsets: &[wgpu::DynamicOffset],
    ) {
        let bind_group = self.arena.bind_groups.alloc(bind_group);
        self.pass.set_bind_group(index, bind_group, offsets);
    }

    /// See [wgpu::RenderPass::set_vertex_buffer].
    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer: Arc<wgpu::Buffer>,
        offset: wgpu::BufferAddress,
    ) {
        let buffer = self.arena.buffers.alloc(buffer);
        self.pass.set_vertex_buffer(slot, buffer.slice(offset..));
    }

    /// See [wgpu::RenderPass::set_index_buffer].
    pub fn set_index_buffer(
        &mut self,
        buffer: Arc<wgpu::Buffer>,
        offset: wgpu::BufferAddress,
        index_format: wgpu::IndexFormat,
    ) {
        let buffer = self.arena.buffers.alloc(buffer);
        self.pass
            .set_index_buffer(buffer.slice(offset..), index_format);
    }
}

impl<'a> std::ops::Deref for ArenaRenderPass<'a> {
    type Target = wgpu::RenderPass<'a>;

    fn deref(&self) -> &Self::Target {
        &self.pass
    }
}

impl<'a> std::ops::DerefMut for ArenaRenderPass<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pass
    }
}
