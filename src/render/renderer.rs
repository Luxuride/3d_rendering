use eframe::egui_wgpu::RenderState;
use eframe::wgpu::util::DeviceExt;
use eframe::{egui, egui_wgpu, wgpu};
use glam::{Mat4, Quat, Vec3};
use std::sync::{Arc, RwLock};
use wgpu::Device;

use crate::render::buffers::camera::Camera;
use crate::render::buffers::camera::camera_raw::CameraRaw;
use crate::render::buffers::texture::texture_raw::TextureRaw;
use crate::render::buffers::transform::Transform;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::model::Model;
use crate::render::model::mesh::axis::{
    x_axis_mesh_builder, y_axis_mesh_builder, z_axis_mesh_builder,
};
use crate::render::model::mesh::cube::cube_mesh_builder;
use crate::render::pipeline::{
    SelectedPipeline, model_pipeline, outline_pipeline, shadow_pipeline, wireframe_pipeline,
};
use eframe::wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, Buffer, RenderPipeline};

const SHADOW_MAP_SIZE: u32 = 2048;
const SHADOW_PARTICIPATION_MIN_Y: f32 = -100.0;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowRaw {
    light_view_proj: [[f32; 4]; 4],
    light_direction: [f32; 4],
    params: [f32; 4],
}

impl ShadowRaw {
    fn new(light_view_proj: Mat4, light_direction: Vec3) -> Self {
        Self {
            light_view_proj: light_view_proj.to_cols_array_2d(),
            light_direction: [light_direction.x, light_direction.y, light_direction.z, 0.0],
            params: [0.0025, 1.0, 0.0, 0.0],
        }
    }
}

pub struct RendererRenderResources {
    wgpu_render_state: RenderState,
    selected_pipeline: SelectedPipeline,

    // Pipelines
    wireframe_pipeline: RenderPipeline,
    model_pipeline: RenderPipeline,
    outline_pipeline: RenderPipeline,
    shadow_pipeline: RenderPipeline,

    // Camera buffer
    camera_bind_group: BindGroup,
    camera_uniform_buffer: Buffer,

    shadow_bind_group: BindGroup,
    shadow_pass_bind_group: BindGroup,
    shadow_uniform_buffer: Buffer,
    shadow_depth_texture: wgpu::Texture,
    shadow_depth_view: wgpu::TextureView,

    // Instances
    outline: Option<Model>,
    axis: [Model; 3],
    models: Vec<Model>,
}

impl RendererRenderResources {
    pub fn new(wgpu_render_state: RenderState, camera: &Camera) -> Self {
        let device = &wgpu_render_state.device;
        let camera_bind_group_layout = Self::camera_bind_group_layout(device);
        let camera_uniform_buffer =
            Self::camera_uniform_buffer(device, camera.get_camera_uniform());
        let camera_bind_group =
            Self::camera_bind_group(device, &camera_bind_group_layout, &camera_uniform_buffer);

        let camera_bind_group_layout = Self::camera_bind_group_layout(device);
        let transform_bind_group_layout = TransformRaw::transform_bind_group_layout(device);
        let texture_bind_group_layout = TextureRaw::diffuse_bind_group_layout(device);
        let shadow_uniform_buffer = Self::shadow_uniform_buffer(device);
        let shadow_pass_bind_group_layout = Self::shadow_pass_bind_group_layout(device);
        let shadow_bind_group_layout = Self::shadow_bind_group_layout(device);
        let (shadow_depth_texture, shadow_depth_view, shadow_depth_sampler) =
            Self::shadow_depth_texture(device);
        let shadow_pass_bind_group = Self::shadow_pass_bind_group(
            device,
            &shadow_pass_bind_group_layout,
            &shadow_uniform_buffer,
        );
        let shadow_bind_group = Self::shadow_bind_group(
            device,
            &shadow_bind_group_layout,
            &shadow_uniform_buffer,
            &shadow_depth_view,
            &shadow_depth_sampler,
        );

        let wireframe_pipeline = wireframe_pipeline(
            device,
            &[
                &camera_bind_group_layout,
                &transform_bind_group_layout,
                &texture_bind_group_layout,
            ],
            wgpu_render_state.target_format.into(),
        );
        let model_pipeline = model_pipeline(
            device,
            &[
                &camera_bind_group_layout,
                &transform_bind_group_layout,
                &texture_bind_group_layout,
                &shadow_bind_group_layout,
            ],
            wgpu_render_state.target_format.into(),
        );
        let outline_pipeline = outline_pipeline(
            device,
            &[
                &camera_bind_group_layout,
                &transform_bind_group_layout,
                &texture_bind_group_layout,
            ],
            wgpu_render_state.target_format.into(),
        );
        let shadow_pipeline = shadow_pipeline(
            device,
            &[&shadow_pass_bind_group_layout, &transform_bind_group_layout],
        );

        let mut models = vec![];
        let cube = cube_mesh_builder().build(device).to_model(
            device,
            &wgpu_render_state.queue,
            (1.0, 1.0, 0.0),
            Transform::default(),
        );
        models.push(cube);

        let axis = [
            x_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (1.0, 0.0, 0.0),
                Transform::default(),
            ),
            y_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (0.0, 1.0, 0.0),
                Transform::default().rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
            ),
            z_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (0.0, 0.0, 1.0),
                Transform::default().rotation(Quat::from_rotation_y((-90.0_f32).to_radians())),
            ),
        ];

        Self {
            wireframe_pipeline,
            model_pipeline,
            outline_pipeline,
            shadow_pipeline,
            camera_bind_group,
            camera_uniform_buffer,
            shadow_bind_group,
            shadow_pass_bind_group,
            shadow_uniform_buffer,
            shadow_depth_texture,
            shadow_depth_view,
            models,
            axis,
            outline: None,
            wgpu_render_state,
            selected_pipeline: SelectedPipeline::Wireframe,
        }
    }

    // Getter methods
    pub fn get_wgpu_render_state(&self) -> &RenderState {
        &self.wgpu_render_state
    }

    pub fn get_selected_pipeline(&self) -> SelectedPipeline {
        self.selected_pipeline
    }

    pub fn get_wireframe_pipeline(&self) -> &RenderPipeline {
        &self.wireframe_pipeline
    }

    pub fn get_model_pipeline(&self) -> &RenderPipeline {
        &self.model_pipeline
    }

    pub fn get_outline_pipeline(&self) -> &RenderPipeline {
        &self.outline_pipeline
    }

    pub fn get_camera_bind_group(&self) -> &BindGroup {
        &self.camera_bind_group
    }

    pub fn get_camera_uniform_buffer(&self) -> &Buffer {
        &self.camera_uniform_buffer
    }

    pub fn get_outline(&self) -> &Option<Model> {
        &self.outline
    }

    pub fn get_axis(&self) -> &[Model; 3] {
        &self.axis
    }

    pub fn get_models(&self) -> &Vec<Model> {
        &self.models
    }

    pub fn get_models_mut(&mut self) -> &mut Vec<Model> {
        &mut self.models
    }

    pub fn get_selected_pipeline_mut(&mut self) -> &mut SelectedPipeline {
        &mut self.selected_pipeline
    }

    pub fn set_outline(&mut self, outline: Option<Model>) {
        self.outline = outline;
    }

    pub fn update_selected_model(&mut self, selected_model: Option<usize>) {
        if let Some(model_idx) = selected_model {
            if model_idx < self.get_models().len() {
                let selected_model = &self.get_models()[model_idx];
                let device = &self.get_wgpu_render_state().device;
                let queue = &self.get_wgpu_render_state().queue;
                let outline_model = selected_model.clone_untextured(device, queue);
                self.set_outline(Some(outline_model));
            }
        } else {
            self.set_outline(None);
        }
    }

    pub fn prepare(&self, _device: &Device, queue: &wgpu::Queue, camera_uniform: CameraRaw) {
        queue.write_buffer(
            self.get_camera_uniform_buffer(),
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        let light_direction = Vec3::new(-0.45, -1.0, -0.35).normalize();
        let light_view_proj = self.light_view_projection(light_direction);
        let shadow_uniform = ShadowRaw::new(light_view_proj, light_direction);
        queue.write_buffer(
            &self.shadow_uniform_buffer,
            0,
            bytemuck::cast_slice(&[shadow_uniform]),
        );
        for model in self.get_models().iter() {
            queue.write_buffer(
                model.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[model.get_transform().to_raw()]),
            );
        }
        if let Some(model) = self.get_outline() {
            queue.write_buffer(
                model.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[model.get_transform().to_raw()]),
            );
        }
        for axis in self.get_axis().iter() {
            queue.write_buffer(
                axis.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[axis.get_transform().to_raw()]),
            );
        }
    }

    pub fn render_shadow_pass(&self, encoder: &mut wgpu::CommandEncoder) {
        let _shadow_depth_guard = &self.shadow_depth_texture;
        let mut shadow_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow_pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.shadow_depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        shadow_render_pass.set_pipeline(&self.shadow_pipeline);
        shadow_render_pass.set_bind_group(0, &self.shadow_pass_bind_group, &[]);

        for model in self.get_models().iter() {
            if Self::model_participates_in_shadows(model) {
                model.draw_depth_only(&mut shadow_render_pass);
            }
        }
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_bind_group(0, self.get_camera_bind_group(), &[]);

        // Render outline
        if let Some(outline) = self.get_outline() {
            render_pass.set_pipeline(self.get_outline_pipeline());
            outline.draw(render_pass);
        }

        // Render models
        match self.get_selected_pipeline() {
            SelectedPipeline::Wireframe => {
                render_pass.set_pipeline(self.get_wireframe_pipeline());
                for model in self.get_models().iter() {
                    model.draw(render_pass);
                }
            }
            SelectedPipeline::Textured => {
                render_pass.set_pipeline(self.get_model_pipeline());
                render_pass.set_bind_group(3, &self.shadow_bind_group, &[]);
                for model in self.get_models().iter() {
                    model.draw(render_pass);
                }
            }
        }

        // Render axis
        render_pass.set_pipeline(self.get_wireframe_pipeline());
        for axis in self.get_axis().iter() {
            axis.draw(render_pass);
        }
    }

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

    fn camera_uniform_buffer(device: &Device, camera_uniform: CameraRaw) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn camera_bind_group(device: &Device, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
    }

    fn shadow_uniform_buffer(device: &Device) -> Buffer {
        let empty = ShadowRaw::new(Mat4::IDENTITY, Vec3::new(-0.45, -1.0, -0.35).normalize());
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow_uniform_buffer"),
            contents: bytemuck::cast_slice(&[empty]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn shadow_pass_bind_group_layout(device: &Device) -> BindGroupLayout {
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
            label: Some("shadow_pass_bind_group_layout"),
        })
    }

    fn shadow_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
            label: Some("shadow_bind_group_layout"),
        })
    }

    fn shadow_depth_texture(device: &Device) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_map_depth_texture"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_map_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        (texture, view, sampler)
    }

    fn shadow_pass_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        shadow_uniform_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: shadow_uniform_buffer.as_entire_binding(),
            }],
            label: Some("shadow_pass_bind_group"),
        })
    }

    fn shadow_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        shadow_uniform_buffer: &Buffer,
        shadow_depth_view: &wgpu::TextureView,
        shadow_depth_sampler: &wgpu::Sampler,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: shadow_uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(shadow_depth_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(shadow_depth_sampler),
                },
            ],
            label: Some("shadow_bind_group"),
        })
    }

    fn light_view_projection(&self, light_direction: Vec3) -> Mat4 {
        let (scene_min, scene_max) = self.scene_bounds().unwrap_or((Vec3::splat(-3.0), Vec3::splat(3.0)));
        let center = (scene_min + scene_max) * 0.5;
        let extent = (scene_max - scene_min).max(Vec3::splat(1.0));
        let radius = extent.length() * 0.5;
        let radius = radius.max(3.0);
        let light_distance = radius * 3.0;
        let light_position = center - light_direction * light_distance;
        let up = if light_direction.dot(Vec3::Y).abs() > 0.95 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let view = Mat4::look_at_rh(light_position, center, up);
        let projection = Mat4::orthographic_rh(
            -radius,
            radius,
            -radius,
            radius,
            0.1,
            radius * 8.0,
        );
        projection * view
    }

    fn scene_bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut scene_min = Vec3::splat(f32::INFINITY);
        let mut scene_max = Vec3::splat(f32::NEG_INFINITY);
        let mut has_bounds = false;

        for model in self.get_models() {
            if Self::model_participates_in_shadows(model)
                && let Some((model_min, model_max)) = model.world_bounds()
            {
                scene_min = scene_min.min(model_min);
                scene_max = scene_max.max(model_max);
                has_bounds = true;
            }
        }

        has_bounds.then_some((scene_min, scene_max))
    }

    fn model_participates_in_shadows(model: &Model) -> bool {
        model.get_transform().get_position().y > SHADOW_PARTICIPATION_MIN_Y
    }
}

pub struct RendererCallback {
    camera_uniform: CameraRaw,
    renderer: Arc<RwLock<RendererRenderResources>>,
}

impl RendererCallback {
    pub fn new(camera_uniform: CameraRaw, renderer: Arc<RwLock<RendererRenderResources>>) -> Self {
        Self {
            camera_uniform,
            renderer,
        }
    }

    // Getter methods
    pub fn get_camera_uniform(&self) -> &CameraRaw {
        &self.camera_uniform
    }

    pub fn get_renderer(&self) -> &Arc<RwLock<RendererRenderResources>> {
        &self.renderer
    }
}

impl egui_wgpu::CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let renderer = &mut self.get_renderer().write().unwrap();
        renderer.prepare(device, queue, *self.get_camera_uniform());
        renderer.render_shadow_pass(egui_encoder);
        vec![]
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let renderer = self.get_renderer().read().unwrap();
        renderer.paint(render_pass);
    }
}
