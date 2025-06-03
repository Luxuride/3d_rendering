use eframe::wgpu;
use eframe::wgpu::{BindGroupLayout, Device};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorRaw {
    model: [f32; 4],
}
impl ColorRaw {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            model: [red, green, blue, alpha],
        }
    }
    pub fn color_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,                               // This binding will hold the TransformationRaw
                visibility: wgpu::ShaderStages::FRAGMENT, // Or VERTEX | FRAGMENT if needed in fragment
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("transform_bind_group_layout"),
        })
    }
    pub fn get_model(&self) -> &[f32; 4] {
        &self.model
    }
}
