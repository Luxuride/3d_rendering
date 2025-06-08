use eframe::wgpu;
use crate::render::buffers::texture::texture_raw;

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture_raw::TextureRaw,
    pub diffuse_bind_group: wgpu::BindGroup,
}
