# 3D Rendering Chess App - Implementation Details

This project is a native Rust desktop app built with eframe/egui and a custom wgpu rendering callback.
It combines:

- Interactive 3D model loading and camera controls
- A custom rendering pipeline (wireframe + textured + outline + shadow pass)
- In-app chess game logic with click-to-move interaction
- Piece animations (jump movement and capture chaos physics)

## Tech Stack

- Rust
- eframe/egui for app shell and UI panels
- egui_wgpu callback for custom rendering
- wgpu for GPU pipelines and draw passes
- glam for math (vectors, matrices, quaternions)
- tobj/image for OBJ + material/texture loading

## Project Structure

- src/main.rs
  - App startup and eframe NativeOptions setup
  - Enables wireframe polygon feature in wgpu

- src/app/
  - app/mod.rs: central app state, input/click handling, chess integration, frame update loop
  - app/input.rs: camera keyboard controls
  - app/panels/: top/center/right/help egui panels

- src/render/
  - renderer.rs: GPU resources, pass orchestration, callback implementation
  - pipeline.rs: render pipeline creation (wireframe/textured/outline/shadow)
  - shader/shader.wgsl: vertex + fragment shaders and shadow sampling
  - model/: mesh/material/model loading and draw methods
  - intersection.rs: ray math for picking
  - animation/: animation trait + concrete animations
  - buffers/: camera/transform/texture/vertex buffer layouts and raw structs

- src/game_logic/chess/
  - state.rs: chess rules, legality, check/checkmate/stalemate
  - scene.rs: chess board <-> model index mapping and click move construction
  - coords.rs: square/world coordinate transforms
  - messages.rs: user-facing move/outcome strings
  - types.rs: core chess enums/structs

- assets/
  - chess.obj and chess.mtl (board + piece templates)

## Runtime Architecture

### 1. App Startup

- main.rs creates a NativeOptions instance with:
  - wgpu renderer
  - depth buffer
  - optional multisampling from environment variable SAMPLE_COUNT
  - required POLYGON_MODE_LINE feature for wireframe pipeline
- Custom3d::new initializes camera and RendererRenderResources in Arc<RwLock<...>>.

### 2. Frame Update Loop

Custom3d::update (eframe::App implementation) performs, each frame:

1. Compute delta time from Instant timestamps
2. Advance active model animations
3. Update capture-chaos despawns
4. Process keyboard input for camera movement
5. Draw top/right/center/help panels
6. Request repaint continuously for real-time rendering

### 3. UI + Custom Render Integration

- center_panel creates an egui canvas-like frame.
- custom_painting registers an egui_wgpu paint callback.
- RendererCallback::prepare:
  - updates camera + shadow uniforms
  - uploads per-model transform buffers
  - records shadow pass
- RendererCallback::paint:
  - draws outline, selected pipeline models, and axis gizmos

## Rendering Implementation

### Pipelines

Defined in render/pipeline.rs:

- Wireframe pipeline
  - PolygonMode::Line
  - Unlit fragment shading
- Textured model pipeline
  - PolygonMode::Fill
  - Lighting + shadow sampling in fs_main
- Outline pipeline
  - Inverted hull style via front-face culling
  - Vertex extrusion in vs_main_outline
- Shadow pipeline
  - Depth-only pass into a dedicated Depth32 shadow map

### Lighting + Shadows

- Directional light vector is normalized and stored in ShadowRaw uniform.
- Scene bounds are estimated dynamically from active model world bounds.
- Light view/projection matrix is rebuilt each frame.
- Shadows are filtered through textureSampleCompare in WGSL.
- Bias is adjusted using N.L to reduce acne.

### Model Representation

Model stores:

- Mesh list and materials
- Transform + GPU uniform buffer + bind group
- Optional boxed animation (dyn Animation + Send + Sync)

Model::get_transform returns animation-derived transform when animation exists.
This allows animation to override rendered transform without immediately mutating the base transform each frame.

## Picking and Selection

Selection is done by ray casting:

1. Convert cursor location to a world ray (screen_to_world_ray)
2. Intersect against all model triangles (Moller-Trumbore)
3. Choose nearest hit
4. Forward to generic model selection or chess click handling

Chess-specific exclusions:

- Ignore capture-chaos models
- Ignore move highlight models

## Chess System Integration

### Loading Chess Scene

load_chess_scene:

1. Loads named OBJ parts
2. Splits board mesh from piece templates using name convention
3. Builds GameState start position
4. Instantiates one model per piece at square-aligned world positions
5. Stores bidirectional mapping:
  - model_by_square (Square -> model index)
  - square_by_model (model index -> Square)
6. Frames camera in top-down orthographic mode

### Name Convention for Piece Templates

parse_piece_template_name expects names like:

- pawn.000 (black)
- pawn.001 (white)
- rook.000, rook.001, etc.

Color suffix mapping:

- .000 -> Black
- .001 -> White

### Click-to-Move Flow

When in chess mode:

1. Click own piece model -> select square
2. Legal destinations are computed and highlighted
3. Click board square -> build Move { from, to }
4. GameState::apply_move validates full legality:
  - piece movement rules
  - occupancy/turn constraints
  - king safety (cannot leave king in check)
  - castling and en passant logic
5. Scene mappings update and moved/captured models are animated
6. Checkmate/stalemate evaluated after each successful move

### Highlights and Capture Handling

- Legal move highlights are small cube models reused and hidden by moving to y = -1000 when not needed.
- Captured piece models are not deleted; they receive ChaosGravityAnimation and are later hidden below scene.

## Animation System

Animation trait:

- progress(delta_time)
- get_animation_transform()
- is_finished()
- blocks_input()

Implemented animations:

- MoveJumpAnimation
  - Parabolic arc between start and destination
  - Input-blocking while active
- ChaosGravityAnimation
  - Ballistic motion with seeded pseudo-random linear/angular velocity
  - Finishes on timeout or off-scene thresholds

## Input and Controls

Keyboard controls:

- W/A/S/D movement
- Space/C vertical movement
- Q/E FOV adjust
- Shift speed boost

Mouse controls:

- Drag controls orientation (perspective) or pan (orthographic)
- Click selects model or issues chess interaction

## Asset Resolution Strategy

resolve_chess_scene_path checks multiple locations in order, including:

- RENDERING_CHESS_OBJ env variable
- cwd-relative assets paths
- APPDIR/APPIMAGE deployment locations
- executable-relative share/assets paths
- fallback to CARGO_MANIFEST_DIR/assets/chess.obj

This supports local dev and packaged app layouts.

## Extending the Project

### Add a New Animation

1. Implement Animation trait in src/render/animation/
2. Attach with model.set_animation(Some(Box::new(...)))
3. Ensure is_finished and blocks_input semantics are correct

### Add New Shader Features

1. Update src/render/shader/shader.wgsl
2. Ensure matching bind groups/layouts in renderer.rs and pipeline.rs
3. Validate both wireframe and textured paths

### Add UI Features

- top_panel: menu/status information
- right_panel: selection, transform edits, chess status
- center_panel: render mode/projection/model import controls

## Build and Run

Standard Cargo:

- cargo run

Optional:

- SAMPLE_COUNT=4 cargo run

Nix dev shell is available via flake.nix for Linux dependency setup.

## Current Limitations

- No undo/redo for chess moves
- Promotions are not surfaced with a user choice flow
- Captured/highlight models are hidden rather than removed/compacted
- Shared model index vectors can fragment over long sessions

## Debugging Tips

- If wireframe fails, verify POLYGON_MODE_LINE feature support on your backend.
- If chess assets fail to load, print and verify resolved path candidates and naming conventions.
- If shadows look unstable, inspect scene bounds and bias constants in ShadowRaw params.
- For picking issues, verify model transforms and whether animation override transforms are expected.
