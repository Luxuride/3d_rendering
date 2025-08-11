use crate::networking::sync::TransformSync;
use crate::render::buffers::camera::{Camera, CameraBuilder};
use crate::render::intersection::screen_to_world_ray;
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use eframe::{egui, egui_wgpu};
use glam::{Vec2, Vec3};
use std::sync::atomic::{AtomicU8, AtomicU64};
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
    last_sync_ms: std::sync::Arc<AtomicU64>,
    peers_count: std::sync::Arc<AtomicU8>,
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
        // Start background networking sync on a separate runtime thread.
        {
            let renderer_clone = renderer.clone();
            let peers_count = std::sync::Arc::new(AtomicU8::new(0));
            let last_sync_ms = std::sync::Arc::new(AtomicU64::new(0));
            let peers_count_clone = peers_count.clone();
            let last_sync_ms_clone = last_sync_ms.clone();
            std::thread::spawn(move || {
                if let Ok(rt) = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    rt.block_on(async move {
                        if let Ok(sync) = TransformSync::new(
                            renderer_clone,
                            peers_count_clone,
                            last_sync_ms_clone,
                        ) {
                            sync.start().await.ok();
                        }
                        futures::future::pending::<()>().await;
                    });
                }
            });
            // store arcs on self after thread spawn
            // They are set in the returned Self below

            // Return Self with arcs
            Some(Self {
                camera,
                renderer,
                selected_model: None,
                loading: Arc::new(AtomicU8::new(0)),
                show_help: false,
                prev_frame: Instant::now(),
                last_sync_ms,
                peers_count,
            })
        }
    }

    // Getter methods
    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_renderer(&self) -> &Arc<RwLock<RendererRenderResources>> {
        &self.renderer
    }

    pub fn get_selected_model(&self) -> Option<usize> {
        self.selected_model
    }

    pub fn get_loading(&self) -> &Arc<AtomicU8> {
        &self.loading
    }

    pub fn get_show_help(&self) -> bool {
        self.show_help
    }

    pub fn get_prev_frame(&self) -> Instant {
        self.prev_frame
    }

    // Setter methods
    pub fn set_selected_model(&mut self, selected_model: Option<usize>) {
        self.selected_model = selected_model;
    }

    pub fn get_selected_model_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_model
    }

    pub fn set_prev_frame(&mut self, prev_frame: Instant) {
        self.prev_frame = prev_frame;
    }
}

impl eframe::App for Custom3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let curr_frame = Instant::now();
        let delta_time: Duration = curr_frame - self.get_prev_frame();
        self.set_prev_frame(curr_frame);
        {
            let mut renderer = self.get_renderer().write().unwrap();
            for model in renderer.get_models_mut().iter_mut() {
                model.add_animation_time(delta_time);
            }
        }
        ctx.input(|i| {
            self.handle_input(i, &delta_time);
        });
        self.top_panel(&delta_time, ctx);
        if self.get_show_help() {
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
        self.get_camera_mut()
            .update_aspect_ratio(rect.width() / rect.height());
        self.get_camera_mut()
            .process_mouse_movement(response.drag_motion().x, response.drag_motion().y);

        if response.clicked() {
            self.handle_model_selection(rect, response.hover_pos());
        }

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RendererCallback::new(
                self.get_camera().get_camera_uniform(),
                self.get_renderer().clone(),
            ),
        ));
    }

    fn handle_model_selection(&mut self, rect: egui::Rect, hover_pos: Option<egui::Pos2>) {
        let Some(pos) = hover_pos else { return };
        let viewport_size = Vec2::new(rect.width(), rect.height());
        let screen_pos = Vec2::new(pos.x - rect.min.x, pos.y - rect.min.y);

        // Validate viewport size
        if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
            return;
        }

        let camera_pos = self.get_camera().get_position();
        let ray_direction = screen_to_world_ray(screen_pos, viewport_size, self.get_camera());

        let mut closest_model: Option<usize> = None;
        let mut closest_distance = f32::INFINITY;

        let renderer = self.get_renderer().read().unwrap();
        for (model_idx, model) in renderer.get_models().iter().enumerate() {
            if let Some(intersection) = model.ray_intersection(camera_pos, ray_direction) {
                let distance = intersection.distance(camera_pos);
                if distance < closest_distance && distance.is_finite() {
                    closest_distance = distance;
                    closest_model = Some(model_idx);
                }
            }
        }
        drop(renderer);

        self.set_selected_model(closest_model);

        let mut renderer = self.get_renderer().write().unwrap();
        renderer.update_selected_model(self.get_selected_model());
    }
}
