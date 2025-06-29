use crate::render::buffers::texture::texture_raw;
use eframe::wgpu;

pub struct Material {
    #[allow(dead_code)]
    diffuse_texture: texture_raw::TextureRaw,
    diffuse_bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(diffuse_texture: &texture_raw::TextureRaw, diffuse_bind_group: wgpu::BindGroup) -> Self {
        Self {
            diffuse_texture: diffuse_texture.clone(),
            diffuse_bind_group,
        }
    }

    // Getter methods
    pub fn get_diffuse_texture(&self) -> &texture_raw::TextureRaw {
        &self.diffuse_texture
    }

    pub fn get_diffuse_bind_group(&self) -> &wgpu::BindGroup {
        &self.diffuse_bind_group
    }
}
