use crate::*;
use std::sync::Arc;
use typed_arena::Arena as TypedArena;

/// Data that exists for the lifetime of a frame.
#[allow(missing_debug_implementations)]
pub struct Frame {
    /// The command encoder for this frame,
    pub cmd: wgpu::CommandEncoder,
    /// Arena for the lifetime of `Frame`.
    pub arena: FrameArena,
}

impl Frame {
    /// Creates a new [Frame].
    pub fn new(cx: &Context) -> Self {
        let cmd = cx.device.create_command_encoder(&Default::default());

        Frame {
            cmd,
            arena: FrameArena::new(),
        }
    }

    /// Submits the frame commands to the queue.
    pub fn submit(self, cx: &Context) {
        cx.queue.submit([self.cmd.finish()]);
    }
}

/// Typed arenas for various GPU resources that need to live as long as the frame.
#[allow(missing_debug_implementations)]
pub struct FrameArena {
    pub render_pipelines: TypedArena<Arc<wgpu::RenderPipeline>>,
    pub bind_groups: TypedArena<Arc<wgpu::BindGroup>>,
    pub buffers: TypedArena<Arc<wgpu::Buffer>>,
}

impl FrameArena {
    /// Creates a new [FrameArena].
    pub fn new() -> Self {
        FrameArena {
            render_pipelines: TypedArena::new(),
            bind_groups: TypedArena::new(),
            buffers: TypedArena::new(),
        }
    }
}
