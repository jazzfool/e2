use crate::*;
use std::sync::Arc;

/// Buffer arena that can grow as needed.
#[derive(Debug)]
pub struct GrowingBufferArena {
    buffers: Vec<(Arc<wgpu::Buffer>, u64)>,
    desc: wgpu::BufferDescriptor<'static>,
}

impl GrowingBufferArena {
    /// Creates a new [GrowingBufferArena].
    ///
    /// The arena will use `desc` to allocate buffers when it grows.
    pub fn new(cx: &Context, desc: wgpu::BufferDescriptor<'static>) -> Self {
        GrowingBufferArena {
            buffers: vec![(Arc::new(cx.device.create_buffer(&desc)), 0)],
            desc,
        }
    }

    /// Allocates a sub-buffer of specified `size`.
    ///
    /// Size *must* be less than `desc.size` (provided with [GrowingBufferArena::new]).
    pub fn allocate(&mut self, cx: &Context, size: u64) -> ArenaAllocation {
        assert!(size <= self.desc.size);

        for (i, (buffer, cursor)) in self.buffers.iter_mut().enumerate() {
            if size <= self.desc.size - *cursor {
                let offset = *cursor;
                *cursor += size;
                return ArenaAllocation {
                    buffer: buffer.clone(),
                    offset,
                    index: i,
                };
            }
        }

        self.grow(cx);
        self.allocate(cx, size)
    }

    /// Resets all the buffer allocations.
    ///
    /// All `ArenaAllocation`s returned from this arena should now be considered invalid.
    pub fn free(&mut self) {
        for (_, cursor) in &mut self.buffers {
            *cursor = 0;
        }
    }

    fn grow(&mut self, cx: &Context) {
        self.buffers
            .push((Arc::new(cx.device.create_buffer(&self.desc)), 0));
    }
}

/// Sub-buffer allocation returned from [GrowingBufferArena].
#[derive(Debug, Clone)]
pub struct ArenaAllocation {
    pub buffer: Arc<wgpu::Buffer>,
    pub offset: u64,
    /// The internal buffer index in [GrowingBufferArena].
    pub index: usize,
}
