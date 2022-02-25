use crate::*;
use crevice::std430::{AsStd430, Std430};
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Draw {
    pub color: Color,
    pub src_rect: Rect,
    pub transform: glam::Mat4,
}

#[derive(AsStd430)]
pub struct GpuDraw {
    pub color: mint::Vector4<f32>,
    pub src_rect: mint::Vector4<f32>,
    pub transform: mint::ColumnMatrix4<f32>,
}

impl From<Draw> for GpuDraw {
    fn from(draw: Draw) -> Self {
        GpuDraw {
            color: mint::Vector4::<f32> {
                x: draw.color.r,
                y: draw.color.g,
                z: draw.color.b,
                w: draw.color.a,
            },
            src_rect: mint::Vector4::<f32> {
                x: draw.src_rect.origin.x,
                y: draw.src_rect.origin.y,
                z: draw.src_rect.origin.x + draw.src_rect.size.x,
                w: draw.src_rect.origin.y + draw.src_rect.size.y,
            },
            transform: draw.transform.into(),
        }
    }
}

static NEXT_DRAW_ARRAY_ID: AtomicU64 = AtomicU64::new(0);

pub struct DrawArray {
    buf: wgpu::Buffer,
    len: usize,
    capacity: u64,
    id: u64,
}

impl DrawArray {
    pub fn new<'a>(cx: &Context, draws: &[Draw]) -> Self {
        let draws = draws
            .iter()
            .map(|&draw| GpuDraw::from(draw).as_std430())
            .collect::<Vec<_>>();
        let size = GpuDraw::std430_size_static() as u64 * draws.len() as u64;
        let buf = cx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: unsafe {
                    std::slice::from_raw_parts(draws.as_ptr() as *const u8, size as _)
                },
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        DrawArray {
            buf,
            len: draws.len(),
            capacity: size,
            id: NEXT_DRAW_ARRAY_ID.fetch_add(1, SeqCst),
        }
    }

    pub fn update(&mut self, cx: &Context, draws: &[Draw]) {
        let draws = draws
            .iter()
            .map(|&draw| GpuDraw::from(draw).as_std430())
            .collect::<Vec<_>>();
        let size = GpuDraw::std430_size_static() as u64 * draws.len() as u64;
        self.len = draws.len();
        if size > self.capacity {
            self.buf = cx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: unsafe {
                        std::slice::from_raw_parts(draws.as_ptr() as *const u8, size as _)
                    },
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });
            self.capacity = size;
            self.id = NEXT_DRAW_ARRAY_ID.fetch_add(1, SeqCst);
        } else {
            cx.queue.write_buffer(&self.buf, 0, unsafe {
                std::slice::from_raw_parts(draws.as_ptr() as *const u8, size as _)
            });
        }
    }

    pub fn set(&self, cx: &Context, at: usize, draw: Draw) {
        assert!(at < self.len);
        let draw = GpuDraw::from(draw).as_std430();
        cx.queue.write_buffer(
            &self.buf,
            at as u64 * GpuDraw::std430_size_static() as u64,
            draw.as_bytes(),
        );
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buf
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }
}
