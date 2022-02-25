use crate::*;
use std::sync::Arc;
use typed_arena::Arena as TypedArena;

pub struct Frame {
    pub cmd: wgpu::CommandEncoder,
    pub arena: FrameArena,
}

impl Frame {
    pub fn new(cx: &Context) -> Self {
        let cmd = cx.device.create_command_encoder(&Default::default());

        Frame {
            cmd,
            arena: FrameArena::new(),
        }
    }

    pub fn submit(self, cx: &Context) {
        cx.queue.submit([self.cmd.finish()]);
    }
}

pub struct FrameArena {
    pub render_pipelines: TypedArena<Arc<wgpu::RenderPipeline>>,
    pub bind_groups: TypedArena<Arc<wgpu::BindGroup>>,
    pub buffers: TypedArena<Arc<wgpu::Buffer>>,
}

impl FrameArena {
    pub fn new() -> Self {
        FrameArena {
            render_pipelines: TypedArena::new(),
            bind_groups: TypedArena::new(),
            buffers: TypedArena::new(),
        }
    }
}
