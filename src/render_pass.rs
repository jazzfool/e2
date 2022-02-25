use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct SimpleRenderPass<'a> {
    pub target: &'a wgpu::TextureView,
    pub resolve: Option<&'a wgpu::TextureView>,
    pub clear: Option<Color>,
}

impl<'a> SimpleRenderPass<'a> {
    pub fn begin(self, frame: &'a mut Frame) -> wgpu::RenderPass<'a> {
        frame.cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: self.target,
                resolve_target: self.resolve,
                ops: wgpu::Operations {
                    load: match self.clear {
                        Some(color) => wgpu::LoadOp::Clear(color.into()),
                        None => wgpu::LoadOp::Load,
                    },
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        })
    }
}
