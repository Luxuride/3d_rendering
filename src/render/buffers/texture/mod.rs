use crate::render::buffers::texture::texture_raw::TextureRaw;
use eframe::wgpu;

pub mod texture_raw;

#[allow(dead_code)] // TODO
pub struct Texture {
    texture_raw: TextureRaw,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(texture_raw: TextureRaw, buffer: wgpu::Buffer, bind_group: wgpu::BindGroup) -> Self {
        Self {
            texture_raw,
            buffer,
            bind_group,
        }
    }

    // Getter methods
    pub fn get_texture_raw(&self) -> &TextureRaw {
        &self.texture_raw
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
