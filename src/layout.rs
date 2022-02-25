use crate::*;
use std::num::{NonZeroU32, NonZeroU64};

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
    pub fn visibility(&self) -> wgpu::ShaderStages {
        match self {
            LayoutEntry::UniformBuffer { visible, .. }
            | LayoutEntry::StorageBuffer { visible, .. }
            | LayoutEntry::Texture { visible, .. }
            | LayoutEntry::Sampler { visible, .. } => *visible,
        }
    }

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

#[derive(Debug, Clone, Copy)]
pub struct BindGroupLayout<'a>(pub &'a [LayoutEntry]);

impl<'a> BindGroupLayout<'a> {
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

#[derive(Debug, Clone, Copy)]
pub struct PipelineLayout<'a>(pub &'a [BindGroupLayout<'a>]);

impl<'a> PipelineLayout<'a> {
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
