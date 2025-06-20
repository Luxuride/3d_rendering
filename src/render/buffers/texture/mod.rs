use crate::render::buffers::texture::texture_raw::TextureRaw;
use eframe::wgpu;

pub mod texture_raw;

#[allow(dead_code)] // TODO
pub struct Texture {
    pub texture_raw: TextureRaw,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
