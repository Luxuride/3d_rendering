use crate::render::buffers::texture::texture_raw::TextureRaw;
use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use eframe::wgpu;
use eframe::wgpu::{
    include_wgsl, BindGroupLayout, ColorTargetState, Device, Face, PolygonMode, RenderPipeline,
};

pub fn model_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
    color_target_state: ColorTargetState,
) -> RenderPipeline {
    pipeline(
        device,
        bind_group_layouts,
        color_target_state,
        PolygonMode::Fill,
        None,
        Some("vs_main"),
    )
}

pub fn wireframe_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
    color_target_state: ColorTargetState,
) -> RenderPipeline {
    pipeline(
        device,
        bind_group_layouts,
        color_target_state,
        PolygonMode::Line,
        None,
        Some("vs_main"),
    )
}

pub fn outline_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
    color_target_state: ColorTargetState,
) -> RenderPipeline {
    pipeline(
        device,
        bind_group_layouts,
        color_target_state,
        PolygonMode::Fill,
        Some(Face::Front), // Inverted Hull Front Face Cull
        Some("vs_main_outline"),
    )
}

fn pipeline_layout<'a>(
    device: &Device,
    bind_group_layouts: &'a [&'a BindGroupLayout],
) -> wgpu::PipelineLayout {
    let mut bind_group_layouts = bind_group_layouts.to_vec();
    let texture_bind_group = TextureRaw::diffuse_bind_group_layout(device);
    bind_group_layouts.push(&texture_bind_group);
    let bind_group_layouts = bind_group_layouts.as_slice();
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline_layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    })
}
fn pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
    color_target_state: ColorTargetState,
    polygon_mode: PolygonMode,
    cull_mode: Option<Face>,
    entry_point: Option<&str>,
) -> RenderPipeline {
    let pipeline_layout = pipeline_layout(device, bind_group_layouts);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("model_shader"),
        source: include_wgsl!("./shader/shader.wgsl").source,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point,
            buffers: &[VertexRaw::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(color_target_state)],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode,
            unclipped_depth: false,
            polygon_mode,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
