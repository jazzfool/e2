use crate::*;
use std::sync::{
    atomic::{AtomicU64, Ordering::SeqCst},
    Arc,
};

static NEXT_SAMPLER_ID: AtomicU64 = AtomicU64::new(0);

/// [wgpu::Sampler] equivalent with a unique ID for use with [BindCache].
#[derive(Debug, Clone)]
pub struct Sampler {
    pub sampler: Arc<wgpu::Sampler>,
    id: u64,
}

impl Sampler {
    /// Creates a new [Sampler] from an existing sampler.
    pub fn new(sampler: Arc<wgpu::Sampler>) -> Self {
        Sampler {
            sampler,
            id: NEXT_SAMPLER_ID.fetch_add(1, SeqCst),
        }
    }

    /// Returns an ID uniquely identifying this [Sampler].
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Simplified sampler descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleSampler {
    pub clamp_u: wgpu::AddressMode,
    pub clamp_v: wgpu::AddressMode,
    pub clamp_w: wgpu::AddressMode,
    pub mag: wgpu::FilterMode,
    pub min: wgpu::FilterMode,
}

impl SimpleSampler {
    /// Sampler with linear filtering and clamped address modes in all directions.
    pub fn linear_clamp() -> Self {
        SimpleSampler {
            clamp_u: wgpu::AddressMode::ClampToEdge,
            clamp_v: wgpu::AddressMode::ClampToEdge,
            clamp_w: wgpu::AddressMode::ClampToEdge,
            mag: wgpu::FilterMode::Linear,
            min: wgpu::FilterMode::Linear,
        }
    }

    /// Sampler with nearest filtering and clamped address modes in all directions.
    ///
    /// Ideal for pixel art.
    pub fn nearest_clamp() -> Self {
        SimpleSampler {
            clamp_u: wgpu::AddressMode::ClampToEdge,
            clamp_v: wgpu::AddressMode::ClampToEdge,
            clamp_w: wgpu::AddressMode::ClampToEdge,
            mag: wgpu::FilterMode::Nearest,
            min: wgpu::FilterMode::Nearest,
        }
    }

    /// Creates a new [Sampler] from the stored sampler configuration.
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
        Sampler::new(Arc::new(sampler))
    }
}
