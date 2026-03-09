use crate::game_logic::chess::{
    ChessSceneState, Color, GameState, ModelMoveUpdate, PieceType, game_outcome_message,
    parse_piece_template_name, square_to_world,
};
use crate::render::buffers::camera::{Camera, CameraBuilder};
use crate::render::buffers::transform::Transform;
use crate::render::intersection::screen_to_world_ray;
use crate::render::model::{Model, NamedModel};
use crate::render::model::mesh::cube::cube_mesh_builder;
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use eframe::{egui, egui_wgpu};
use glam::{Vec2, Vec3};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    chess_state: Option<ChessSceneState>,
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
            chess_state: None,
        })
    }

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

    pub fn get_chess_state(&self) -> Option<&ChessSceneState> {
        self.chess_state.as_ref()
    }

    pub fn set_selected_model(&mut self, selected_model: Option<usize>) {
        self.selected_model = selected_model;
    }

    pub fn get_selected_model_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_model
    }

    pub fn set_prev_frame(&mut self, prev_frame: Instant) {
        self.prev_frame = prev_frame;
    }

    pub fn resolve_chess_scene_path() -> Result<PathBuf, String> {
        let mut candidates: Vec<PathBuf> = Vec::new();

        if let Ok(explicit) = std::env::var("RENDERING_CHESS_OBJ") {
            candidates.push(PathBuf::from(explicit));
        }

        if let Ok(cwd) = std::env::current_dir() {
            candidates.extend([
                cwd.join("assets").join("chess.obj"),
                cwd.join("chess.obj"),
                cwd.join("src").join("chess.obj"),
            ]);
        }

        if let Ok(appdir) = std::env::var("APPDIR") {
            candidates.push(
                Path::new(&appdir)
                    .join("usr")
                    .join("share")
                    .join("rendering")
                    .join("chess.obj"),
            );
        }

        if let Ok(appimage) = std::env::var("APPIMAGE")
            && let Some(appimage_dir) = Path::new(&appimage).parent()
        {
            candidates.extend([
                appimage_dir
                    .join("..")
                    .join("share")
                    .join("rendering")
                    .join("chess.obj"),
                appimage_dir
                    .join("..")
                    .join("usr")
                    .join("share")
                    .join("rendering")
                    .join("chess.obj"),
                appimage_dir.join("assets").join("chess.obj"),
            ]);
        }

        if let Ok(exe) = std::env::current_exe()
            && let Some(exe_dir) = exe.parent()
        {
            candidates.extend([
                exe_dir.join("chess.obj"),
                exe_dir.join("assets").join("chess.obj"),
                exe_dir.join("..").join("assets").join("chess.obj"),
                exe_dir
                    .join("..")
                    .join("..")
                    .join("assets")
                    .join("chess.obj"),
                exe_dir
                    .join("..")
                    .join("share")
                    .join("rendering")
                    .join("chess.obj"),
                exe_dir
                    .join("..")
                    .join("..")
                    .join("share")
                    .join("rendering")
                    .join("chess.obj"),
            ]);
        }

        candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/chess.obj"));
        candidates.push(PathBuf::from("assets/chess.obj"));
        candidates.push(PathBuf::from("src/chess.obj"));

        let mut tried: Vec<String> = Vec::new();
        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
            tried.push(candidate.display().to_string());
        }

        Err(format!(
            "Could not find chess.obj. Set RENDERING_CHESS_OBJ or place the file in one of these locations: {}",
            tried.join(", ")
        ))
    }

    pub fn load_chess_scene(&mut self, file_path: &Path) -> Result<(), String> {
        let (device, queue) = {
            let renderer = self
                .get_renderer()
                .read()
                .map_err(|_| "Renderer lock poisoned")?;
            (
                renderer.get_wgpu_render_state().device.clone(),
                renderer.get_wgpu_render_state().queue.clone(),
            )
        };

        let named_models =
            Model::load_named_models(file_path, &device, &queue, Transform::default())
                .map_err(|err| format!("Failed to load chess model: {err}"))?;

        let mut board_parts: Vec<Model> = Vec::new();
        let mut piece_template_parts: HashMap<(PieceType, Color), Vec<Model>> = HashMap::new();

        for NamedModel { name, model } in named_models {
            if name.eq_ignore_ascii_case("board") {
                board_parts.push(model);
                continue;
            }

            if let Some(key) = parse_piece_template_name(&name) {
                piece_template_parts.entry(key).or_default().push(model);
            }
        }

        let board_template =
            merge_models(&device, board_parts).ok_or("Missing `board` object in chess.obj")?;
        let piece_templates = piece_template_parts
            .into_iter()
            .filter_map(|(key, parts)| merge_models(&device, parts).map(|model| (key, model)))
            .collect::<HashMap<_, _>>();

        let chess_state = {
            let mut renderer = self
                .get_renderer()
                .write()
                .map_err(|_| "Renderer lock poisoned")?;
            renderer.get_models_mut().clear();

            let board_model_index = renderer.get_models().len();
            renderer.get_models_mut().push(board_template);

            let (board_min, board_max) = renderer.get_models()[board_model_index]
                .world_bounds()
                .ok_or("Board object has no vertices")?;

            let game_state = GameState::new_start_position();
            let mut model_by_square = HashMap::new();
            let mut square_by_model = HashMap::new();

            for (square, piece) in game_state.iter_pieces() {
                let Some(template) = piece_templates.get(&(piece.piece_type, piece.color)) else {
                    return Err(format!(
                        "Missing mesh for {:?} {:?}. Ensure object names use piece.000/.001",
                        piece.color, piece.piece_type
                    ));
                };

                let mut transform = Transform::default();
                transform.set_position(square_to_world(square, board_min, board_max));
                let instance = template.instance_with_transform(&device, transform);

                let model_index = renderer.get_models().len();
                renderer.get_models_mut().push(instance);
                model_by_square.insert(square, model_index);
                square_by_model.insert(model_index, square);
            }

            ChessSceneState::new(
                game_state,
                board_model_index,
                board_min,
                board_max,
                model_by_square,
                square_by_model,
            )
        };

        self.set_selected_model(None);
        if let Ok(mut renderer) = self.get_renderer().write() {
            renderer.update_selected_model(None);
        }
        self.chess_state = Some(chess_state);

        Ok(())
    }

    pub fn import_chess_scene(&mut self) -> Result<(), String> {
        let path = Self::resolve_chess_scene_path()?;
        self.load_chess_scene(&path)
    }

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

        if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
            return;
        }

        let camera_pos = self.get_camera().get_position();
        let ray_direction = screen_to_world_ray(screen_pos, viewport_size, self.get_camera());

        let mut closest_model: Option<usize> = None;
        let mut closest_intersection: Option<Vec3> = None;
        let mut closest_distance = f32::INFINITY;

        let renderer = self.get_renderer().read().unwrap();
        for (model_idx, model) in renderer.get_models().iter().enumerate() {
            if self
                .chess_state
                .as_ref()
                .is_some_and(|state| state.is_highlight_model(model_idx))
            {
                continue;
            }

            if let Some(intersection) = model.ray_intersection(camera_pos, ray_direction) {
                let distance = intersection.distance(camera_pos);
                if distance < closest_distance && distance.is_finite() {
                    closest_distance = distance;
                    closest_model = Some(model_idx);
                    closest_intersection = Some(intersection);
                }
            }
        }
        drop(renderer);

        if self.chess_state.is_some() {
            self.handle_chess_click(closest_model, closest_intersection);
            return;
        }

        self.set_selected_model(closest_model);
        let mut renderer = self.get_renderer().write().unwrap();
        renderer.update_selected_model(self.get_selected_model());
    }

    fn handle_chess_click(&mut self, closest_model: Option<usize>, hit_point: Option<Vec3>) {
        let Some(mut chess_state) = self.chess_state.take() else {
            return;
        };

        if chess_state.game_outcome.is_some() {
            self.set_selected_model(None);
            if let Ok(mut renderer) = self.get_renderer().write() {
                clear_move_highlights(&mut chess_state, &mut renderer);
                renderer.update_selected_model(None);
            }
            chess_state.clear_selection();
            self.chess_state = Some(chess_state);
            return;
        }

        chess_state.clear_last_error();

        if let Some(model_index) = closest_model {
            if chess_state.try_select_piece_model(model_index).is_some() {
                self.set_selected_model(Some(model_index));
                if let Ok(mut renderer) = self.get_renderer().write() {
                    update_move_highlights(&mut chess_state, &mut renderer);
                    renderer.update_selected_model(self.get_selected_model());
                }
                self.chess_state = Some(chess_state);
                return;
            }

            if let Some(chess_move) = chess_state.try_build_click_move(model_index, hit_point) {
                match chess_state.game_state.apply_move(chess_move) {
                    Ok(()) => {
                        let update =
                            chess_state.apply_mapping_after_move(chess_move.from, chess_move.to);
                        if let Ok(mut renderer) = self.get_renderer().write() {
                            apply_move_to_models(update, &mut renderer);
                            clear_move_highlights(&mut chess_state, &mut renderer);
                            renderer.update_selected_model(None);
                        }

                        let side_to_move = chess_state.game_state.side_to_move();
                        if chess_state.game_state.is_checkmate(side_to_move) {
                            let winner = side_to_move.opposite();
                            let outcome = crate::game_logic::chess::GameOutcome::Checkmate {
                                winner,
                            };
                            chess_state.game_outcome = Some(outcome);
                            chess_state.last_error = Some(game_outcome_message(outcome));
                        } else if chess_state.game_state.is_stalemate(side_to_move) {
                            let outcome = crate::game_logic::chess::GameOutcome::Stalemate;
                            chess_state.game_outcome = Some(outcome);
                            chess_state.last_error = Some(game_outcome_message(outcome));
                        }

                        chess_state.clear_selection();
                        self.set_selected_model(None);
                    }
                    Err(err) => {
                        chess_state.last_error =
                            Some(crate::game_logic::chess::move_error_message(err));
                    }
                }

                if let Ok(mut renderer) = self.get_renderer().write() {
                    update_move_highlights(&mut chess_state, &mut renderer);
                }

                self.chess_state = Some(chess_state);
                return;
            }
        }

        chess_state.clear_selection();
        self.set_selected_model(None);
        if let Ok(mut renderer) = self.get_renderer().write() {
            clear_move_highlights(&mut chess_state, &mut renderer);
            renderer.update_selected_model(None);
        }
        self.chess_state = Some(chess_state);
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

fn apply_move_to_models(update: Option<ModelMoveUpdate>, renderer: &mut RendererRenderResources) {
    let Some(update) = update else {
        return;
    };

    if let Some(captured_model_index) = update.captured_model_index
        && let Some(model) = renderer.get_models_mut().get_mut(captured_model_index)
    {
        model
            .get_transform_mut()
            .set_position(Vec3::new(0.0, -1000.0, 0.0));
    }

    if let Some(model) = renderer.get_models_mut().get_mut(update.moving_model_index) {
        model
            .get_transform_mut()
            .set_position(update.destination_world_position);
    }
}

fn update_move_highlights(chess_state: &mut ChessSceneState, renderer: &mut RendererRenderResources) {
    let Some(from) = chess_state.selected_square else {
        clear_move_highlights(chess_state, renderer);
        return;
    };

    let legal_targets = chess_state.game_state.legal_moves_from(from);
    let board_min = chess_state.board_min;
    let board_max = chess_state.board_max;
    let square_width = (board_max.x - board_min.x) / 8.0;
    let square_depth = (board_max.z - board_min.z) / 8.0;

    let highlight_scale = Vec3::new(square_width * 0.65, 0.04, square_depth * 0.65);

    let (device, queue) = (
        renderer.get_wgpu_render_state().device.clone(),
        renderer.get_wgpu_render_state().queue.clone(),
    );

    for (idx, square) in legal_targets.iter().copied().enumerate() {
        let world = chess_state.square_to_world(square);

        if idx >= chess_state.highlight_model_indices.len() {
            let mut transform = Transform::default();
            transform.set_position(Vec3::new(world.x, board_max.y + 0.02, world.z));
            *transform.get_scale_mut() = highlight_scale;

            let highlight_model = cube_mesh_builder().build(&device).to_model(
                &device,
                &queue,
                (0.15, 0.9, 0.25),
                transform,
            );

            let model_index = renderer.get_models().len();
            renderer.get_models_mut().push(highlight_model);
            chess_state.highlight_model_indices.push(model_index);
        } else if let Some(model) = renderer
            .get_models_mut()
            .get_mut(chess_state.highlight_model_indices[idx])
        {
            model
                .get_transform_mut()
                .set_position(Vec3::new(world.x, board_max.y + 0.02, world.z));
            *model.get_transform_mut().get_scale_mut() = highlight_scale;
        }
    }

    for hidden_idx in legal_targets.len()..chess_state.highlight_model_indices.len() {
        if let Some(model) = renderer
            .get_models_mut()
            .get_mut(chess_state.highlight_model_indices[hidden_idx])
        {
            model
                .get_transform_mut()
                .set_position(Vec3::new(0.0, -1000.0, 0.0));
        }
    }
}

fn clear_move_highlights(chess_state: &mut ChessSceneState, renderer: &mut RendererRenderResources) {
    for model_index in chess_state.highlight_model_indices.iter().copied() {
        if let Some(model) = renderer.get_models_mut().get_mut(model_index) {
            model
                .get_transform_mut()
                .set_position(Vec3::new(0.0, -1000.0, 0.0));
        }
    }
}

fn merge_models(device: &eframe::wgpu::Device, models: Vec<Model>) -> Option<Model> {
    let first = models.first()?;
    let mut meshes = Vec::new();
    for model in &models {
        meshes.extend_from_slice(model.get_meshes());
    }
    Some(Model::new(
        device,
        meshes,
        first.get_materials().to_vec(),
        Transform::default(),
    ))
}
