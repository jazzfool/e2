use crate::*;

pub use wgpu::TextureViewDimension::*;

/// Simplified render pipeline descriptor.
#[derive(Debug, Clone)]
pub struct SimpleRenderPipeline<'a> {
    pub layout: Option<&'a wgpu::PipelineLayout>,
    pub vertex: &'a wgpu::ShaderModule,
    pub fragment: &'a wgpu::ShaderModule,
    pub vertex_entry: &'a str,
    pub fragment_entry: &'a str,
    pub vertex_layout: VertexLayout<'a>,
    pub samples: u32,
    pub format: wgpu::TextureFormat,
    pub blend: Option<wgpu::BlendState>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
}

impl<'a> SimpleRenderPipeline<'a> {
    /// Creates a new [wgpu::RenderPipeline] from the stored pipeline configuration.
    pub fn create(self, cx: &Context) -> wgpu::RenderPipeline {
        cx.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: self.layout,
                vertex: wgpu::VertexState {
                    module: self.vertex,
                    entry_point: self.vertex_entry,
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: self.vertex_layout.stride,
                        step_mode: self.vertex_layout.step_mode,
                        attributes: &self
                            .vertex_layout
                            .attributes
                            .iter()
                            .enumerate()
                            .map(|(i, &attr)| wgpu::VertexAttribute {
                                shader_location: i as _,
                                ..attr.into()
                            })
                            .collect::<Vec<_>>(),
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: self.depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: self.samples,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: self.fragment,
                    entry_point: self.fragment_entry,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.format,
                        blend: self.blend,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
            })
    }
}
