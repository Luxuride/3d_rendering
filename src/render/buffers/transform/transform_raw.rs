use eframe::wgpu;
use eframe::wgpu::{BindGroupLayout, Device};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    model: [[f32; 4]; 4],
}

impl TransformRaw {
    pub fn new(model: [[f32; 4]; 4]) -> Self {
        Self { model }
    }

    pub fn get_model(&self) -> &[[f32; 4]; 4] {
        &self.model
    }

    pub fn transform_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,                             // This binding will hold the TransformationRaw
                visibility: wgpu::ShaderStages::VERTEX, // Or VERTEX | FRAGMENT if needed in fragment
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
}
