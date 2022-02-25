use crate::*;
use crevice::std430::AsStd430;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct InstanceBuffer {
    pub buffer: Arc<wgpu::Buffer>,
    pub size: u64,
    pub free: bool,
}

/// [BatchRenderer] can draw many items with the same mesh and texture efficiently.
///
/// It does this by uploading the batch of draw data into a buffer and issuing a single instanced draw call.
///
/// Even more efficiently, [BatchRenderer] can pull from draw data from a [DrawArray].
#[derive(Debug)]
pub struct BatchRenderer {
    instances: Vec<InstanceBuffer>,
    instance_desc: wgpu::BufferDescriptor<'static>,

    storage_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,

    storage_binds: BindCache,
    texture_binds: BindCache,
    sampler_binds: BindCache,

    storage_slot: u32,
    texture_slot: u32,
    sampler_slot: u32,
}

impl BatchRenderer {
    /// Creates a new [BatchRenderer].
    ///
    /// The renderer is not necessarily tied to [BatchRenderPipeline].
    /// The pipeline handle only acts a reference pipeline layout.
    pub fn new(pipeline: &BatchRenderPipeline) -> Self {
        let instance_desc = wgpu::BufferDescriptor {
            label: None,
            size: 0,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let storage_layout = pipeline.pipeline.get_bind_group_layout(0);
        let texture_layout = pipeline.pipeline.get_bind_group_layout(1);
        let sampler_layout = pipeline.pipeline.get_bind_group_layout(2);

        BatchRenderer {
            instances: vec![],
            instance_desc,

            storage_layout,
            texture_layout,
            sampler_layout,

            storage_binds: BindCache::new(),
            texture_binds: BindCache::new(),
            sampler_binds: BindCache::new(),

            storage_slot: 0,
            texture_slot: 1,
            sampler_slot: 2,
        }
    }

    /// Sets the binding slots for the renderer.
    ///
    /// Generally you should not call this directly, but instead call it through
    /// a pipeline type.
    ///
    /// For example, [BatchRenderer::bind] will automatically call this.
    pub fn bind(&mut self, storage: u32, texture: u32, sampler: u32) {
        self.storage_slot = storage;
        self.texture_slot = texture;
        self.sampler_slot = sampler;
    }

    /// Resets the previously allocated buffers, making them available for reuse.
    ///
    /// Call this at the start or end of every frame in order to maintain acceptable spatial performance.
    pub fn free(&mut self) {
        for buf in &mut self.instances {
            buf.free = true;
        }
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

    /// Draws a specified mesh and texture multiple times.
    ///
    /// The draw is instanced `draws.len()` times, and each draw uses the corresponding `Draw`.
    ///
    /// Prefer [BatchRenderer::draw_array] where possible.
    pub fn draw(
        &mut self,
        cx: &Context,
        pass: &mut ArenaRenderPass,
        mesh: &Mesh,
        texture: &Texture,
        draws: &[Draw],
    ) {
        let size = GpuDraw::std430_size_static() as u64 * draws.len() as u64;
        let (index, buf) = if let Some((i, buf)) = self
            .instances
            .iter_mut()
            .enumerate()
            .filter(|(_, x)| x.free && x.size >= size)
            .min_by(|(_, x), (_, y)| x.size.cmp(&y.size))
        {
            buf.free = false;
            (i, buf.buffer.clone())
        } else {
            let buffer = Arc::new(cx.device.create_buffer(&wgpu::BufferDescriptor {
                size,
                ..self.instance_desc
            }));
            self.instances.push(InstanceBuffer {
                buffer: buffer.clone(),
                size,
                free: false,
            });
            (self.instances.len() - 1, buffer)
        };

        let draws = draws
            .iter()
            .map(|&draw| GpuDraw::from(draw).as_std430())
            .collect::<Vec<_>>();
        cx.queue.write_buffer(buf.as_ref(), 0, unsafe {
            std::slice::from_raw_parts(draws.as_ptr() as *const u8, size as _)
        });

        let storage_group = self.storage_binds.get(
            cx,
            index as _,
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.storage_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: buf.as_ref(),
                        offset: 0,
                        size: None,
                    }),
                }],
            },
        );

        let texture_group = self.texture_binds.get(
            cx,
            texture.id(),
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                }],
            },
        );

        pass.set_bind_group(self.storage_slot, storage_group, &[]);
        pass.set_bind_group(self.texture_slot, texture_group, &[]);

        pass.set_vertex_buffer(0, mesh.vertices.clone(), 0);
        pass.set_index_buffer(mesh.indices.clone(), 0, wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..mesh.index_count as u32, 0, 0..draws.len() as u32);
    }

    /// Functions identically to [BatchRenderer::draw], except the
    /// draw data is read from a [DrawArray].
    ///
    /// This is generally far more preferable in terms of temporal performance.
    pub fn draw_array(
        &mut self,
        cx: &Context,
        pass: &mut ArenaRenderPass,
        mesh: &Mesh,
        texture: &Texture,
        array: &DrawArray,
    ) {
        let storage_group = self.storage_binds.get(
            cx,
            array.id(),
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.storage_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: array.buffer(),
                        offset: 0,
                        size: None,
                    }),
                }],
            },
        );

        let texture_group = self.texture_binds.get(
            cx,
            texture.id(),
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                }],
            },
        );

        pass.set_bind_group(self.storage_slot, storage_group, &[]);
        pass.set_bind_group(self.texture_slot, texture_group, &[]);

        pass.set_vertex_buffer(0, mesh.vertices.clone(), 0);
        pass.set_index_buffer(mesh.indices.clone(), 0, wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..mesh.index_count as u32, 0, 0..array.len() as u32);
    }
}

/// A simple 2D render pipeline designed for use with [BatchRenderer].
#[derive(Debug, Clone)]
pub struct BatchRenderPipeline {
    pub layout: Arc<wgpu::PipelineLayout>,
    pub pipeline: Arc<wgpu::RenderPipeline>,
}

impl BatchRenderPipeline {
    /// Creates a new [BatchRenderPipeline] with the given parameters.
    pub fn new(
        cx: &Context,
        samples: u32,
        format: wgpu::TextureFormat,
        blend: Option<wgpu::BlendState>,
    ) -> Self {
        let (layout, _) = PipelineLayout(&[
            BindGroupLayout(&[LayoutEntry::StorageBuffer {
                visible: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
                dynamic_offset: false,
                min_binding_size: None,
                read_only: true,
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
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/batch.wgsl").into()),
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
        }
        .create(cx);

        BatchRenderPipeline {
            layout: Arc::new(layout),
            pipeline: Arc::new(pipeline),
        }
    }

    /// Bind the pipeline and renderer to a given render pass.
    pub fn bind(&self, pass: &mut ArenaRenderPass, batch: &mut BatchRenderer) {
        pass.set_pipeline(self.pipeline.clone());
        batch.bind(0, 1, 2);
    }
}
