use crate::*;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

static NEXT_SAMPLER_ID: AtomicU64 = AtomicU64::new(0);

pub struct Sampler {
    pub sampler: wgpu::Sampler,
    id: u64,
}

impl Sampler {
    pub fn new(sampler: wgpu::Sampler) -> Self {
        Sampler {
            sampler,
            id: NEXT_SAMPLER_ID.fetch_add(1, SeqCst),
        }
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }
}

pub struct SimpleSampler {
    pub clamp_u: wgpu::AddressMode,
    pub clamp_v: wgpu::AddressMode,
    pub clamp_w: wgpu::AddressMode,
    pub mag: wgpu::FilterMode,
    pub min: wgpu::FilterMode,
}

impl SimpleSampler {
    pub fn linear_clamp() -> Self {
        SimpleSampler {
            clamp_u: wgpu::AddressMode::ClampToEdge,
            clamp_v: wgpu::AddressMode::ClampToEdge,
            clamp_w: wgpu::AddressMode::ClampToEdge,
            mag: wgpu::FilterMode::Linear,
            min: wgpu::FilterMode::Linear,
        }
    }

    pub fn nearest_clamp() -> Self {
        SimpleSampler {
            clamp_u: wgpu::AddressMode::ClampToEdge,
            clamp_v: wgpu::AddressMode::ClampToEdge,
            clamp_w: wgpu::AddressMode::ClampToEdge,
            mag: wgpu::FilterMode::Nearest,
            min: wgpu::FilterMode::Nearest,
        }
    }

    pub fn create(self, cx: &Context) -> Sampler {
        let sampler = cx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: self.clamp_u,
            address_mode_v: self.clamp_v,
            address_mode_w: self.clamp_w,
            mag_filter: self.mag,
            min_filter: self.min,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.,
            lod_max_clamp: 1.,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });
        Sampler::new(sampler)
    }
}
