use crate::*;

pub struct Frame {
    pub cmd: wgpu::CommandEncoder,
}

impl Frame {
    pub fn new(cx: &Context) -> Self {
        let cmd = cx.device.create_command_encoder(&Default::default());

        Frame { cmd }
    }

    pub fn submit(self, cx: &Context) {
        cx.queue.submit([self.cmd.finish()]);
    }
}
