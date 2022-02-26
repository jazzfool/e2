use crate::*;
use std::num::{NonZeroU32, NonZeroU64};

/// Simplified bind group layout entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutEntry {
    UniformBuffer {
        visible: wgpu::ShaderStages,
        count: Option<NonZeroU32>,
        dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
    },
    StorageBuffer {
        visible: wgpu::ShaderStages,
        count: Option<NonZeroU32>,
        dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
        read_only: bool,
    },
    Texture {
        visible: wgpu::ShaderStages,
        count: Option<NonZeroU32>,
        ty: wgpu::TextureSampleType,
        dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    },
    Sampler {
        visible: wgpu::ShaderStages,
        count: Option<NonZeroU32>,
        comparison: bool,
    },
}

impl LayoutEntry {
    /// Returns shader stage visibility.
    pub fn visibility(&self) -> wgpu::ShaderStages {
        match self {
            LayoutEntry::UniformBuffer { visible, .. }
            | LayoutEntry::StorageBuffer { visible, .. }
            | LayoutEntry::Texture { visible, .. }
            | LayoutEntry::Sampler { visible, .. } => *visible,
        }
    }

    /// Returns entry count.
    pub fn count(&self) -> Option<NonZeroU32> {
        match self {
            LayoutEntry::UniformBuffer { count, .. }
            | LayoutEntry::StorageBuffer { count, .. }
            | LayoutEntry::Texture { count, .. }
            | LayoutEntry::Sampler { count, .. } => *count,
        }
    }
}

impl From<LayoutEntry> for wgpu::BindGroupLayoutEntry {
    fn from(entry: LayoutEntry) -> Self {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: entry.visibility(),
            ty: match entry {
                LayoutEntry::UniformBuffer {
                    dynamic_offset,
                    min_binding_size,
                    ..
                } => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: dynamic_offset,
                    min_binding_size,
                },
                LayoutEntry::StorageBuffer {
                    dynamic_offset,
                    min_binding_size,
                    read_only,
                    ..
                } => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: dynamic_offset,
                    min_binding_size,
                },
                LayoutEntry::Texture {
                    ty,
                    dimension,
                    multisampled,
                    ..
                } => wgpu::BindingType::Texture {
                    sample_type: ty,
                    view_dimension: dimension,
                    multisampled,
                },
                LayoutEntry::Sampler { comparison, .. } => {
                    wgpu::BindingType::Sampler(if comparison {
                        wgpu::SamplerBindingType::Comparison
                    } else {
                        wgpu::SamplerBindingType::Filtering
                    })
                }
            },
            count: entry.count(),
        }
    }
}

/// Simplified bind group layout descriptor.
#[derive(Debug, Clone, Copy)]
pub struct BindGroupLayout<'a>(pub &'a [LayoutEntry]);

impl<'a> BindGroupLayout<'a> {
    /// Creates a new [wgpu::BindGroupLayout] from stored entries.
    pub fn create(self, cx: &Context) -> wgpu::BindGroupLayout {
        cx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &self
                    .0
                    .iter()
                    .enumerate()
                    .map(|(i, &entry)| wgpu::BindGroupLayoutEntry {
                        binding: i as u32,
                        ..entry.into()
                    })
                    .collect::<Vec<_>>(),
            })
    }
}

/// Simplified pipeline layout descriptor.
#[derive(Debug, Clone, Copy)]
pub struct PipelineLayout<'a>(pub &'a [BindGroupLayout<'a>]);

impl<'a> PipelineLayout<'a> {
    /// Creates a new [wgpu::PipelineLayout] from the stored bind group layouts.
    ///
    /// Also returns, at tuple index 1, the [wgpu::BindGroupLayout]s created in the process.
    pub fn create(self, cx: &Context) -> (wgpu::PipelineLayout, Vec<wgpu::BindGroupLayout>) {
        let groups = self
            .0
            .iter()
            .map(|&group| group.create(cx))
            .collect::<Vec<_>>();

        let layout = cx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &groups.iter().collect::<Vec<_>>(),
                push_constant_ranges: &[],
            });

        (layout, groups)
    }
}

/// Simplified vertex attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttribute {
    Float { offset: u64 },
    Vec2 { offset: u64 },
    Vec3 { offset: u64 },
    Vec4 { offset: u64 },
}

impl VertexAttribute {
    /// Returns the offset of the attribute in the vertex.
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

/// Describes the memory layout of a vertex buffer.
#[derive(Debug, Clone, Copy)]
pub struct VertexLayout<'a> {
    /// Bytes between elements of the vertex buffer.
    pub stride: u64,
    /// How to step across the vertex buffer.
    pub step_mode: wgpu::VertexStepMode,
    /// Attributes (position, UV, etc) of the vertex layout.
    pub attributes: &'a [VertexAttribute],
}
