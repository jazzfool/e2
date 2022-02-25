use crate::*;
use std::{collections::HashMap, sync::Arc};

pub struct BindCache {
    cache: HashMap<u64, Arc<wgpu::BindGroup>>,
}

impl BindCache {
    pub fn new() -> Self {
        BindCache {
            cache: HashMap::new(),
        }
    }

    pub fn get(
        &mut self,
        cx: &Context,
        key: u64,
        or_insert: &wgpu::BindGroupDescriptor,
    ) -> &Arc<wgpu::BindGroup> {
        self.cache
            .entry(key)
            .or_insert_with(|| Arc::new(cx.device.create_bind_group(or_insert)))
    }
}
