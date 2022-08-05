use crate::*;
use std::sync::Arc;

/// GPU vertex with position and UV.
#[repr(C)]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

impl Vertex {
    /// Vertex layout compatible with this vertex type.
    pub fn layout() -> VertexLayout<'static> {
        static ATTRIBUTES: [VertexAttribute; 2] = [
            VertexAttribute::Vec2 { offset: 0 },
            VertexAttribute::Vec2 { offset: 8 },
        ];

        VertexLayout {
            stride: std::mem::size_of::<Self>() as _,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Mesh stored as vertex and index buffers.
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Arc<wgpu::Buffer>,
    pub indices: Arc<wgpu::Buffer>,
    pub vertex_capacity: u64,
    pub index_capacity: u64,
    pub vertex_count: u64,
    pub index_count: u64,
}

impl Mesh {
    /// Creates a new [Mesh] initialized with `vertices` and `indices`.
    pub fn new(cx: &Context, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vb = Self::create_vb(cx, vertices.len() as _);
        cx.queue.write_buffer(&vb, 0, unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>(),
            )
        });

        let ib = Self::create_ib(cx, indices.len() as _);
        cx.queue.write_buffer(&ib, 0, unsafe {
            std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4)
        });

        Mesh {
            vertices: Arc::new(vb),
            indices: Arc::new(ib),
            vertex_capacity: vertices.len() as _,
            index_capacity: indices.len() as _,
            vertex_count: vertices.len() as _,
            index_count: indices.len() as _,
        }
    }

    /// Sets new vertices.
    pub fn set_vertices(&mut self, cx: &Context, vertices: &[Vertex]) {
        self.vertex_count = vertices.len() as _;
        if self.vertex_count > self.vertex_capacity {
            self.vertices = Arc::new(Self::create_vb(cx, self.vertex_count));
            self.vertex_capacity = self.vertex_count;
        }
        cx.queue.write_buffer(&self.vertices, 0, unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>(),
            )
        });
    }

    /// Sets new indices.
    pub fn set_indices(&mut self, cx: &Context, indices: &[u32]) {
        self.index_count = indices.len() as _;
        if self.index_count > self.index_capacity {
            self.indices = Arc::new(Self::create_ib(cx, self.index_count));
            self.index_capacity = self.index_count;
        }
        cx.queue.write_buffer(&self.indices, 0, unsafe {
            std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4)
        });
    }

    fn create_vb(cx: &Context, count: u64) -> wgpu::Buffer {
        cx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Vertex>() as u64 * count,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_ib(cx: &Context, count: u64) -> wgpu::Buffer {
        cx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4 * count,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
