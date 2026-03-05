use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use eframe::wgpu;
use eframe::wgpu::{
    BindGroupLayout, ColorTargetState, Device, Face, MultisampleState, PolygonMode, RenderPipeline,
    include_wgsl,
};
use std::env;
use std::sync::LazyLock;

pub static SAMPLE_COUNT: LazyLock<u16> = LazyLock::new(|| {
    env::var("SAMPLE_COUNT")
        .map(|x| x.parse::<u16>().ok())
        .ok()
        .flatten()
        .unwrap_or(1)
});

#[derive(PartialEq, Clone, Copy)]
pub enum SelectedPipeline {
    Wireframe,
    Textured,
}

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
        Some(Face::Back),
        Some("vs_main"),
        Some("fs_main"),
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
        Some("fs_unlit"),
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
        Some("fs_unlit"),
    )
}

pub fn shadow_pipeline(device: &Device, bind_group_layouts: &[&BindGroupLayout]) -> RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow_pipeline_layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shadow_shader"),
        source: include_wgsl!("./shader/shader.wgsl").source,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_shadow"),
            buffers: &[VertexRaw::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: MultisampleState {
            count: 1,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}

fn pipeline_layout(device: &Device, bind_group_layouts: &[&BindGroupLayout]) -> wgpu::PipelineLayout {
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
    fragment_entry_point: Option<&str>,
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
            entry_point: fragment_entry_point,
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
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: MultisampleState {
            count: (*SAMPLE_COUNT).into(),
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}
