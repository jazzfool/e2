use crate::*;
use std::{
    borrow::Cow,
    num::NonZeroU32,
    path::Path,
    sync::atomic::{AtomicU64, Ordering::SeqCst},
};

static NEXT_TEXTURE_ID: AtomicU64 = AtomicU64::new(0);

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    id: u64,
}

impl Texture {
    pub fn new(texture: wgpu::Texture, view: wgpu::TextureView) -> Self {
        Texture {
            texture,
            view,
            id: NEXT_TEXTURE_ID.fetch_add(1, SeqCst),
        }
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }
}

pub struct ImageTexture<'a> {
    pub format: wgpu::TextureFormat,
    pub pixels: Cow<'a, [u8]>,
    pub width: u32,
    pub height: u32,
}

impl<'a> ImageTexture<'a> {
    pub fn from_path(path: impl AsRef<Path>, srgb: bool) -> Result<Self> {
        let image = image::open(path)?.into_rgba8();
        let (width, height) = (image.width(), image.height());
        let raw = image.into_raw();
        Ok(ImageTexture {
            format: if srgb {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            pixels: Cow::Owned(raw),
            width,
            height,
        })
    }

    pub fn from_image<P: image::Pixel<Subpixel = u8>>(
        image: &'a image::ImageBuffer<P, Vec<u8>>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let (width, height) = (image.width(), image.height());
        let raw = image.as_raw();
        ImageTexture {
            format,
            pixels: Cow::Borrowed(raw),
            width,
            height,
        }
    }

    pub fn create(self, cx: &Context) -> Texture {
        let texture = cx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        cx.queue.write_texture(
            texture.as_image_copy(),
            self.pixels.as_ref(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    NonZeroU32::new(self.format.describe().block_size as u32 * self.width).unwrap(),
                ),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        Texture::new(texture, view)
    }
}

pub struct RenderTexture {
    pub format: wgpu::TextureFormat,
    pub samples: u32,
    pub width: u32,
    pub height: u32,
    pub binding: bool,
}

impl RenderTexture {
    pub fn create(self, cx: &Context) -> Texture {
        let texture = cx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.samples,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: if self.binding {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            },
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture::new(texture, view)
    }
}
