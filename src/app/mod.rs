use crate::render::buffers::camera::{Camera, CameraBuilder};
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use cgmath::Point3;
use eframe::{egui, egui_wgpu};
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

mod input;
pub mod panels;

pub struct Custom3d {
    camera: Camera,
    renderer: Arc<RwLock<RendererRenderResources>>,
    selected_model: SelectedModel,
    loading: Arc<AtomicU8>,
    show_help: bool,
    prev_frame: Instant,
}

#[derive(PartialEq, Clone, Copy)]
enum SelectedModel {
    Wireframe(usize),
    Model(usize),
    None,
}

impl Custom3d {
    pub fn new(cc: &eframe::CreationContext) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.clone()?;
        let camera = CameraBuilder::default()
            .z_far(500.0)
            .position(Point3::new(0.0, 0.0, -5.0))
            .build();

        let renderer = Arc::new(RwLock::new(RendererRenderResources::new(
            wgpu_render_state,
            &camera,
        )));
        Some(Self {
            camera,
            renderer,
            selected_model: SelectedModel::None,
            loading: Arc::new(AtomicU8::new(0)),
            show_help: false,
            prev_frame: Instant::now(),
        })
    }
}

impl eframe::App for Custom3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let curr_frame = Instant::now();
        let delta_time: Duration = curr_frame - self.prev_frame;
        self.prev_frame = curr_frame;
        {
            let mut renderer = self.renderer.write().unwrap();
            for model in renderer.wireframe_models.iter_mut() {
                model.add_animation_time(delta_time);
            }
            for model in renderer.models.iter_mut() {
                model.add_animation_time(delta_time);
            }
        }
        ctx.input(|i| {
            self.handle_input(i, &delta_time);
        });
        self.top_panel(&delta_time, ctx);
        if self.show_help {
            self.help(ctx);
        }
        self.right_panel(ctx);
        self.center_panel(ctx);
        ctx.request_repaint();
    }
}

impl Custom3d {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            egui::Sense::drag(),
        );
        self.camera
            .update_aspect_ratio(rect.width() / rect.height());
        self.camera
            .process_mouse_movement(response.drag_motion().x, response.drag_motion().y);
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RendererCallback::new(self.camera.get_camera_uniform(), self.renderer.clone()),
        ));
    }
}
