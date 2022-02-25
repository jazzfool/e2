use crate::*;

pub use wgpu::TextureViewDimension::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttribute {
    Float { offset: u64 },
    Vec2 { offset: u64 },
    Vec3 { offset: u64 },
    Vec4 { offset: u64 },
}

impl VertexAttribute {
    pub fn offset(&self) -> u64 {
        match self {
            VertexAttribute::Float { offset }
            | VertexAttribute::Vec2 { offset }
            | VertexAttribute::Vec3 { offset }
            | VertexAttribute::Vec4 { offset } => *offset,
        }
    }
}

impl From<VertexAttribute> for wgpu::VertexFormat {
    fn from(attr: VertexAttribute) -> Self {
        match attr {
            VertexAttribute::Float { .. } => wgpu::VertexFormat::Float32,
            VertexAttribute::Vec2 { .. } => wgpu::VertexFormat::Float32x2,
            VertexAttribute::Vec3 { .. } => wgpu::VertexFormat::Float32x3,
            VertexAttribute::Vec4 { .. } => wgpu::VertexFormat::Float32x4,
        }
    }
}

impl From<VertexAttribute> for wgpu::VertexAttribute {
    fn from(attr: VertexAttribute) -> Self {
        wgpu::VertexAttribute {
            format: attr.into(),
            offset: attr.offset(),
            shader_location: 0,
        }
    }
}

pub struct VertexLayout<'a> {
    pub stride: u64,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: &'a [VertexAttribute],
}

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
}

impl<'a> SimpleRenderPipeline<'a> {
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
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: self.samples,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: self.fragment,
                    entry_point: self.fragment_entry,
                    targets: &[wgpu::ColorTargetState {
                        format: self.format,
                        blend: self.blend,
                        write_mask: wgpu::ColorWrites::all(),
                    }],
                }),
                multiview: None,
            })
    }
}
