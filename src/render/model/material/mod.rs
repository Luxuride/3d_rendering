use eframe::wgpu;

pub mod texture;

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
}
