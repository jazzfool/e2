use crate::*;
use crevice::std430::AsStd430;
use std::{num::NonZeroU64, sync::Arc};

/// Simplified interface for drawing a mesh and texture
/// with a specified "draw configuration" ([MeshDraw]).
#[derive(Debug)]
pub struct MeshRenderer {
    uniforms: GrowingBufferArena,

    uniform_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,

    uniform_binds: BindCache,
    texture_binds: BindCache,
    sampler_binds: BindCache,

    uniform_slot: u32,
    texture_slot: u32,
    sampler_slot: u32,
}

impl MeshRenderer {
    /// Creates a new [MeshRenderer].
    ///
    /// The renderer is not necessarily tied to [MeshRenderPipeline].
    /// The pipeline handle only acts a reference pipeline layout.
    pub fn new(cx: &Context, pipeline: &MeshRenderPipeline) -> Self {
        let uniforms = GrowingBufferArena::new(
            cx,
            wgpu::BufferDescriptor {
                label: None,
                size: cx.pad_uniform_size(GpuDraw::std430_size_static() as _) * 1024,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        let uniform_layout = pipeline.pipeline.get_bind_group_layout(0);
        let texture_layout = pipeline.pipeline.get_bind_group_layout(1);
        let sampler_layout = pipeline.pipeline.get_bind_group_layout(2);

        MeshRenderer {
            uniforms,

            uniform_layout,
            texture_layout,
            sampler_layout,

            uniform_binds: BindCache::new(),
            texture_binds: BindCache::new(),
            sampler_binds: BindCache::new(),

            uniform_slot: 0,
            texture_slot: 1,
            sampler_slot: 2,
        }
    }

    /// Resets the previously allocated buffers, making them available for reuse.
    ///
    /// Call this at the start or end of every frame in order to maintain acceptable spatial performance.
    pub fn reset(&mut self) {
        self.uniforms.reset();
    }

    /// Binds a sampler for use with the proceeding draw calls.
    pub fn bind_sampler<'a>(
        &mut self,
        cx: &Context,
        pass: &'a mut wgpu::RenderPass,
        sampler: &Sampler,
    ) {
        let group = self.sampler_binds.get(
            cx,
            sampler.id(),
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.sampler_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler.sampler),
                }],
            },
        );

        pass.set_bind_group(
            self.sampler_slot,
            unsafe { &*(group.as_ref() as *const _) },
            &[],
        );
    }

    /// Draws a textured mesh with `draw` parameters.
    pub fn draw(&mut self, cx: &Context, pass: &mut ArenaRenderPass, draw: MeshDraw) {
        let alloc = self
            .uniforms
            .allocate(cx, cx.pad_uniform_size(GpuDraw::std430_size_static() as _));

        let gpu_draw = GpuDraw::from(&draw);
        cx.queue.write_buffer(
            alloc.buffer.as_ref(),
            alloc.offset,
            gpu_draw.as_std430().as_bytes(),
        );

        let uniform_group = self.uniform_binds.get(
            cx,
            alloc.index as _,
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.uniform_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: alloc.buffer.as_ref(),
                        offset: 0,
                        size: Some(NonZeroU64::new(GpuDraw::std430_size_static() as _).unwrap()),
                    }),
                }],
            },
        );

        let texture_group = self.texture_binds.get(
            cx,
            draw.texture.id(),
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&draw.texture.view),
                }],
            },
        );

        pass.set_bind_group(self.uniform_slot, uniform_group, &[alloc.offset as u32]);
        pass.set_bind_group(self.texture_slot, texture_group, &[]);

        pass.set_vertex_buffer(0, draw.mesh.vertices.clone(), 0);
        pass.set_index_buffer(draw.mesh.indices.clone(), 0, wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..draw.mesh.index_count as u32, 0, 0..1);
    }
}

/// Implemented by any non-batching renderer with a 3 slot design (uniform, texture, sampler).
pub trait Slot3MeshRenderer {
    /// Sets the binding slots for the renderer.
    ///
    /// Generally you should not call this directly, but instead call it through
    /// a pipeline type.
    ///
    /// For example, [MeshRenderer::bind] will automatically call this.
    fn bind(&mut self, uniform: u32, texture: u32, sampler: u32);
}

impl Slot3MeshRenderer for MeshRenderer {
    fn bind(&mut self, uniform: u32, texture: u32, sampler: u32) {
        self.uniform_slot = uniform;
        self.texture_slot = texture;
        self.sampler_slot = sampler;
    }
}

/// A simple 2D render pipeline designed for use with [MeshRenderer].
#[derive(Debug, Clone)]
pub struct MeshRenderPipeline {
    pub layout: Arc<wgpu::PipelineLayout>,
    pub pipeline: Arc<wgpu::RenderPipeline>,
}

impl MeshRenderPipeline {
    /// Creates a new [MeshRenderPipeline] with the given parameters.
    pub fn new(
        cx: &Context,
        samples: u32,
        format: wgpu::TextureFormat,
        blend: Option<wgpu::BlendState>,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let (layout, _) = PipelineLayout(&[
            BindGroupLayout(&[LayoutEntry::UniformBuffer {
                visible: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
                dynamic_offset: true,
                min_binding_size: Some(
                    NonZeroU64::new(GpuDraw::std430_size_static() as _).unwrap(),
                ),
            }]),
            BindGroupLayout(&[LayoutEntry::Texture {
                visible: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::TextureSampleType::Float { filterable: true },
                dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            }]),
            BindGroupLayout(&[LayoutEntry::Sampler {
                visible: wgpu::ShaderStages::FRAGMENT,
                count: None,
                comparison: false,
            }]),
        ])
        .create(cx);

        let shader = cx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/mesh.wgsl").into()),
            });

        let pipeline = SimpleRenderPipeline {
            layout: Some(&layout),
            vertex: &shader,
            fragment: &shader,
            vertex_entry: "vs_main",
            fragment_entry: "fs_main",
            vertex_layout: Vertex::layout(),
            samples,
            format,
            blend,
            depth_stencil,
        }
        .create(cx);

        MeshRenderPipeline {
            layout: Arc::new(layout),
            pipeline: Arc::new(pipeline),
        }
    }

    /// Bind the pipeline and renderer to a given render pass.
    pub fn bind(&self, pass: &mut ArenaRenderPass, renderer: &mut impl Slot3MeshRenderer) {
        pass.set_pipeline(self.pipeline.clone());
        renderer.bind(0, 1, 2);
    }
}
