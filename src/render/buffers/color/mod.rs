use crate::render::buffers::color::color_raw::ColorRaw;
use eframe::wgpu;

pub mod color_raw;

#[allow(dead_code)] // TODO
pub struct Color {
    color: ColorRaw,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
