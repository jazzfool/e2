use crate::*;
use crevice::std430::AsStd430;
use std::sync::Arc;

#[derive(Clone)]
struct InstanceBuffer {
    pub buffer: Arc<wgpu::Buffer>,
    pub size: u64,
    pub free: bool,
}

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

    pub fn bind(&mut self, storage: u32, texture: u32, sampler: u32) {
        self.storage_slot = storage;
        self.texture_slot = texture;
        self.sampler_slot = sampler;
    }

    pub fn free(&mut self) {
        for buf in &mut self.instances {
            buf.free = true;
        }
    }

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

    pub fn draw<'a, 'b>(
        &'a mut self,
        cx: &Context,
        pass: &'a mut wgpu::RenderPass<'b>,
        mesh: &'b Mesh,
        texture: &'b Texture,
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

        pass.set_bind_group(
            self.storage_slot,
            unsafe { &*(storage_group.as_ref() as *const _) },
            &[],
        );
        pass.set_bind_group(
            self.texture_slot,
            unsafe { &*(texture_group.as_ref() as *const _) },
            &[],
        );

        pass.set_vertex_buffer(0, mesh.vertices.slice(..));
        pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..mesh.index_count as u32, 0, 0..draws.len() as u32);
    }

    pub fn draw_array<'a, 'b>(
        &'a mut self,
        cx: &Context,
        pass: &'a mut wgpu::RenderPass<'b>,
        mesh: &'b Mesh,
        texture: &'b Texture,
        array: &'b DrawArray,
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

        pass.set_bind_group(
            self.storage_slot,
            unsafe { &*(storage_group.as_ref() as *const _) },
            &[],
        );
        pass.set_bind_group(
            self.texture_slot,
            unsafe { &*(texture_group.as_ref() as *const _) },
            &[],
        );

        pass.set_vertex_buffer(0, mesh.vertices.slice(..));
        pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..mesh.index_count as u32, 0, 0..array.len() as u32);
    }
}

pub struct BatchRenderPipeline {
    pub layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl BatchRenderPipeline {
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

        BatchRenderPipeline { layout, pipeline }
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, batch: &mut BatchRenderer) {
        pass.set_pipeline(&self.pipeline);
        batch.bind(0, 1, 2);
    }
}
