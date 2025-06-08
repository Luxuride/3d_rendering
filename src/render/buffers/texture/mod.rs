use eframe::wgpu;
use crate::render::buffers::texture::texture_raw::TextureRaw;

pub mod texture_raw;

pub struct Texture {
    pub texture_raw: TextureRaw,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}