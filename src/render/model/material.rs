use crate::render::buffers::texture::texture_raw;
use eframe::wgpu;

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture_raw::TextureRaw,
    pub diffuse_bind_group: wgpu::BindGroup,
}
