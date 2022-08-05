use crate::*;
use crevice::std430::AsStd430;
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering::SeqCst},
};
use wgpu::util::DeviceExt;

/// Specifies state for a single mesh draw.
///
/// This should be created on the fly and **not** stored.
#[derive(Debug, Clone)]
pub struct MeshDraw<'a> {
    /// Mesh to be drawn
    pub mesh: &'a Mesh,
    /// Texture to be sampled.
    pub texture: &'a Texture,
    /// Color to multiply texture color with.
    /// Using [Color::WHITE] will mean the texture will render as-is.
    pub color: Color,
    /// UV sub-rectangle to use.
    /// Using [Rect::ONE] will mean that the full UV space is available.
    pub src_rect: Rect,
    /// Global transform to use.
    pub transform: glam::Mat4,
}

impl<'a> From<&'a MeshDraw<'a>> for GpuDraw {
    fn from(draw: &'a MeshDraw) -> Self {
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

/// Specifies state for a batched mesh draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatchDraw {
    /// Color to multiply texture color with.
    /// Using [Color::WHITE] will mean the texture will render as-is.
    pub color: Color,
    /// UV sub-rectangle to use.
    /// Using [Rect::ONE] will mean that the full UV space is available.
    pub src_rect: Rect,
    /// Global transform to use.
    pub transform: glam::Mat4,
}

impl From<BatchDraw> for GpuDraw {
    fn from(draw: BatchDraw) -> Self {
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

/// Stores the same data as [BatchDraw], but in a GPU-friendly manner.
///
/// When uploading, convert to `Std430` first with [crevice::std430::AsStd430].
#[derive(AsStd430, Debug, Clone, Copy, PartialEq)]
pub struct GpuDraw {
    pub color: mint::Vector4<f32>,
    pub src_rect: mint::Vector4<f32>,
    pub transform: mint::ColumnMatrix4<f32>,
}

static NEXT_DRAW_ARRAY_ID: AtomicU64 = AtomicU64::new(0);

/// An efficient draw data buffer for use with batched renderers.
///
/// This allows for instance data to persist across frames, including
/// methods to conservatively update the instance data.
#[derive(Debug)]
pub struct DrawArray<D: Into<GpuDraw>> {
    buf: wgpu::Buffer,
    len: usize,
    capacity: u64,
    id: u64,
    _marker: PhantomData<D>,
}

impl<D: Clone + Into<GpuDraw>> DrawArray<D> {
    /// Creates a new [DrawArray] initialized with `draws`.
    pub fn new(cx: &Context, draws: impl Iterator<Item = D>) -> Self {
        let draws = draws
            .map(|draw| draw.clone().into().as_std430())
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
            _marker: PhantomData,
        }
    }

    /// Completely updates the draw data.
    pub fn update(&mut self, cx: &Context, draws: impl Iterator<Item = D>) {
        let draws = draws
            .map(|draw| draw.clone().into().as_std430())
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

    /// Updates a single instance at the index `at`.
    pub fn set(&self, cx: &Context, at: usize, draw: D) {
        assert!(at < self.len);
        let draw = draw.into().as_std430();
        cx.queue.write_buffer(
            &self.buf,
            at as u64 * GpuDraw::std430_size_static() as u64,
            draw.as_bytes(),
        );
    }

    /// Returns the buffer containing draw data.
    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buf
    }

    /// Returns the number instances.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the *capacity* of the draw array.
    ///
    /// This is how many more bytes of draw data it can store before needing buffer recreation.
    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns an ID uniquely identifying this [DrawArray].
    ///
    /// Primarily for use with [BindCache].
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }
}
