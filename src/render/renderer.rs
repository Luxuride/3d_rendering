use cgmath::InnerSpace;
use cgmath::Zero;
use eframe::egui_wgpu::RenderState;
use eframe::wgpu::util::DeviceExt;
use eframe::{egui, egui_wgpu, wgpu};
use std::sync::{Arc, RwLock};
use wgpu::Device;

use crate::camera::camera::Camera;
use crate::camera::camera_uniform::CameraUniform;
use crate::render::model::vertex::vertex_raw::{VertexRaw};
use eframe::wgpu::{
    include_wgsl, BindGroup, BindGroupEntry, BindGroupLayout, Buffer, ColorTargetState,
    RenderPipeline,
};
use crate::render::model::mesh::{Mesh, MeshBuilder};
use crate::render::model::transformation::Transformation;
use crate::render::model::transformation::transformation_raw::TransformationRaw;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

const VERTICES: &[VertexRaw] = &[
    VertexRaw {
        position: [-0.0868241, 0.49240386, 0.0],
    }, // A
    VertexRaw {
        position: [-0.49513406, 0.06958647, 0.0],
    }, // B
    VertexRaw {
        position: [-0.21918549, -0.44939706, 0.0],
    }, // C
    VertexRaw {
        position: [0.35966998, -0.3473291, 0.0],
    }, // D
    VertexRaw {
        position: [0.44147372, 0.2347359, 0.0],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub struct RendererRenderResources {
    pub pipeline: RenderPipeline,
    // Camera buffer
    camera_bind_group: BindGroup,
    camera_uniform_buffer: Buffer,

    // Instance
    instances: Vec<Mesh>,
}

impl RendererRenderResources {
    pub fn new(device: &Device, wgpu_render_state: &RenderState, camera: &Camera) -> Self {
        let camera_bind_group_layout = Self::camera_bind_group_layout(device);
        let camera_uniform_buffer =
            Self::camera_uniform_buffer(device, camera.get_camera_uniform());
        let camera_bind_group =
            Self::camera_bind_group(device, &camera_bind_group_layout, &camera_uniform_buffer);

        let camera_bind_group_layout = Self::camera_bind_group_layout(device);
        let transformation_bind_group_layout = TransformationRaw::transform_bind_group_layout(device);
        let pipeline_layout = Self::pipeline_layout(device, &[&camera_bind_group_layout, &transformation_bind_group_layout]);
        let pipeline = Self::pipeline(
            device,
            pipeline_layout,
            wgpu_render_state.target_format.into(),
        );

        let mut instances = vec![];
        let mesh1 = MeshBuilder::new().vertices(VERTICES.to_vec()).indices(INDICES.to_vec()).build(device);
        instances.push(mesh1);

        Self {
            pipeline,
            camera_bind_group,
            camera_uniform_buffer,
            instances,
        }
    }

    pub fn prepare(&self, _device: &Device, queue: &wgpu::Queue, camera_uniform: CameraUniform) {
        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        for instance in self.instances.iter() {
            queue.write_buffer(instance.get_transformation_buffer(), 0, bytemuck::cast_slice(&[instance.get_transformation().to_raw()]));
        }
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for instance in self.instances.iter() {
            render_pass.set_bind_group(1, instance.get_transformation_bind_group(), &[]); // Bind transformation to slot 1

            render_pass.set_vertex_buffer(0, instance.get_vertex_buffer().slice(..));
            render_pass.set_index_buffer(instance.get_index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..instance.get_num_indices(), 0, 0..1); // Draw 1 instance of this mesh
        }
    }
}

pub struct RendererCallback {
    camera_uniform: CameraUniform,
    renderer: Arc<RwLock<RendererRenderResources>>,
}

impl RendererCallback {
    pub fn new(
        camera_uniform: CameraUniform,
        renderer: Arc<RwLock<RendererRenderResources>>,
    ) -> Self {
        Self {
            camera_uniform,
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
            .prepare(device, queue, self.camera_uniform);
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

impl RendererRenderResources {
    fn camera_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        })
    }
    fn camera_uniform_buffer(device: &Device, camera_uniform: CameraUniform) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
    fn camera_bind_group(
        device: &Device,
        camera_bind_group_layout: &BindGroupLayout,
        camera_uniform_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
    }

    fn pipeline_layout<'a>(
        device: &Device,
        bind_group_layouts: &'a [&'a BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        })
    }

    fn pipeline(
        device: &Device,
        pipeline_layout: wgpu::PipelineLayout,
        color_target_state: ColorTargetState,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: include_wgsl!("./shader/shader.wgsl").source,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
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
                topology: wgpu::PrimitiveTopology::TriangleStrip, // Draw as lines for wireframe
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
        })
    }
}
