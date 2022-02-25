use crate::*;

/// Stores GPU context handles, most notably the device and queue.
#[derive(Debug)]
pub struct Context {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Context {
    /// Creates a new [Context], rendering to `window`, selecting a backend from `backends`.
    pub fn new(
        window: &impl raw_window_handle::HasRawWindowHandle,
        backends: wgpu::Backends,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .ok_or_else(|| Error::NoSuitableAdapter)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))?;

        Ok(Context {
            instance,
            surface,
            adapter,
            device,
            queue,
        })
    }

    /// Configures the surface with `width` and `height` in pixels and with `present_mode` presentation.
    pub fn configure_surface(&self, width: u32, height: u32, present_mode: wgpu::PresentMode) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface.get_preferred_format(&self.adapter).unwrap(),
                width,
                height,
                present_mode,
            },
        );
    }

    /// Helper function for [wgpu::Surface::get_current_texture]
    pub fn next_frame(&self) -> Result<wgpu::SurfaceTexture> {
        Ok(self.surface.get_current_texture()?)
    }

    /// Helper function to pad uniform sizes to the next multiple of the minimum uniform buffer alignment.
    pub fn pad_uniform_size(&self, size: u64) -> u64 {
        let min = self.device.limits().min_uniform_buffer_offset_alignment as u64;
        if min > 0 {
            (size + min - 1) & !(min - 1)
        } else {
            size
        }
    }
}
