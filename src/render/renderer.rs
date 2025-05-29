use eframe::egui_wgpu::RenderState;
use eframe::wgpu::util::DeviceExt;
use eframe::{egui, egui_wgpu, wgpu};
use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};
use wgpu::Device;

use cgmath::{Matrix4, SquareMatrix};
use eframe::wgpu::ShaderSource;

pub struct RendererRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
    pub model_matrix: Matrix4<f32>,
    num_vertices: u32,
}

impl RendererRenderResources {
    pub fn new(
        device: &Device,
        wgpu_render_state: &RenderState,
        num_vertices: u32,
        source: ShaderSource,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(std::mem::size_of::<Matrix4<f32>>() as u64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[], // No vertex buffers for position, generated in shader
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu_render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList, // Draw as lines for wireframe
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for wireframe
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                // ENABLE DEPTH STENCIL
                format: wgpu::TextureFormat::Depth32Float, // Choose a suitable depth format
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // Draw if new depth is less than existing
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(<Matrix4<f32> as AsRef<[f32; 16]>>::as_ref(
                &Matrix4::identity(),
            )),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let model_matrix = Matrix4::identity();

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            model_matrix,
            num_vertices,
        }
    }

    pub fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_projection_matrix: Matrix4<f32>,
    ) {
        let transform_matrix = view_projection_matrix * self.model_matrix;

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(<Matrix4<f32> as AsRef<[f32; 16]>>::as_ref(
                &transform_matrix,
            )),
        );
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..self.num_vertices, 0..1);
    }
}

pub struct RendererCallback {
    view_projection_matrix: Matrix4<f32>,
    renderer: Arc<RwLock<RendererRenderResources>>,
}

impl RendererCallback {
    pub fn new(
        view_projection_matrix: Matrix4<f32>,
        renderer: Arc<RwLock<RendererRenderResources>>,
    ) -> Self {
        Self {
            view_projection_matrix,
            renderer,
        }
    }
}

impl egui_wgpu::CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.renderer
            .read()
            .unwrap()
            .prepare(device, queue, self.view_projection_matrix);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _: &egui_wgpu::CallbackResources,
    ) {
        self.renderer.read().unwrap().paint(render_pass);
    }
}
