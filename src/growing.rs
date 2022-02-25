use crate::*;
use std::sync::Arc;

pub struct GrowingBufferArena {
    buffers: Vec<(Arc<wgpu::Buffer>, u64)>,
    desc: wgpu::BufferDescriptor<'static>,
}

impl GrowingBufferArena {
    pub fn new(cx: &Context, desc: wgpu::BufferDescriptor<'static>) -> Self {
        GrowingBufferArena {
            buffers: vec![(Arc::new(cx.device.create_buffer(&desc)), 0)],
            desc,
        }
    }

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

pub struct ArenaAllocation {
    pub buffer: Arc<wgpu::Buffer>,
    pub offset: u64,
    pub index: usize,
}
