use crate::render::buffers::camera::{Camera, CameraBuilder};
use crate::render::intersection::screen_to_world_ray;
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use eframe::{egui, egui_wgpu};
use glam::{Vec2, Vec3};
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

mod input;
pub mod panels;

pub struct Custom3d {
    camera: Camera,
    renderer: Arc<RwLock<RendererRenderResources>>,
    selected_model: Option<usize>,
    loading: Arc<AtomicU8>,
    show_help: bool,
    prev_frame: Instant,
}

impl Custom3d {
    pub fn new(cc: &eframe::CreationContext) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.clone()?;
        let camera = CameraBuilder::default()
            .z_far(500.0)
            .position(Vec3::new(0.0, 0.0, -5.0))
            .build();

        let renderer = Arc::new(RwLock::new(RendererRenderResources::new(
            wgpu_render_state,
            &camera,
        )));
        Some(Self {
            camera,
            renderer,
            selected_model: None,
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
            egui::Sense::click_and_drag(),
        );
        self.camera
            .update_aspect_ratio(rect.width() / rect.height());
        self.camera
            .process_mouse_movement(response.drag_motion().x, response.drag_motion().y);

        if response.clicked() {
            self.handle_model_selection(rect, response.hover_pos());
        }

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RendererCallback::new(self.camera.get_camera_uniform(), self.renderer.clone()),
        ));
    }

    fn handle_model_selection(&mut self, rect: egui::Rect, hover_pos: Option<egui::Pos2>) {
        if let Some(pos) = hover_pos {
            let viewport_size = Vec2::new(rect.width(), rect.height());
            let screen_pos = Vec2::new(pos.x - rect.min.x, pos.y - rect.min.y);

            // Validate viewport size
            if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
                return;
            }

            let ray_direction = screen_to_world_ray(screen_pos, viewport_size, &self.camera);

            let mut closest_model: Option<usize> = None;
            let mut closest_distance = f32::INFINITY;

            let renderer = self.renderer.read().unwrap();
            for (model_idx, model) in renderer.models.iter().enumerate() {
                if let Some(intersection) =
                    model.ray_intersection(self.camera.get_position(), ray_direction)
                {
                    let distance = intersection.distance(self.camera.get_position());
                    if distance < closest_distance && distance.is_finite() {
                        closest_distance = distance;
                        closest_model = Some(model_idx);
                    }
                }
            }

            self.selected_model = closest_model;

            drop(renderer);
            let mut renderer = self.renderer.write().unwrap();
            renderer.update_selected_model(self.selected_model);
        }
    }
}
