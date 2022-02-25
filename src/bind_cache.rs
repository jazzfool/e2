use crate::*;
use std::{collections::HashMap, sync::Arc};

/// Caches bind groups basd on a `u64` key.
#[derive(Debug, Clone)]
pub struct BindCache {
    cache: HashMap<u64, Arc<wgpu::BindGroup>>,
}

impl BindCache {
    /// Creates a new [BindCache].
    pub fn new() -> Self {
        BindCache {
            cache: HashMap::new(),
        }
    }

    /// Either return the bind group at `key`, or if it does not exist,
    /// a new bind group is created using `or_insert` and inserted at `key`, then returned.
    pub fn get(
        &mut self,
        cx: &Context,
        key: u64,
        or_insert: &wgpu::BindGroupDescriptor,
    ) -> Arc<wgpu::BindGroup> {
        self.cache
            .entry(key)
            .or_insert_with(|| Arc::new(cx.device.create_bind_group(or_insert)))
            .clone()
    }
}
