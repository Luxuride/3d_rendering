use crate::render::buffers::texture::texture_raw;
use eframe::wgpu;

pub struct Material {
    #[allow(dead_code)]
    pub diffuse_texture: texture_raw::TextureRaw,
    pub diffuse_bind_group: wgpu::BindGroup,
}
