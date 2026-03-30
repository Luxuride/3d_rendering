use crate::render::animation::Animation;
use crate::render::buffers::transform::Transform;
use glam::Vec3;
use std::time::Duration;

const MOVE_JUMP_DURATION_SECONDS: f32 = 0.38;
const MOVE_JUMP_BASE_HEIGHT: f32 = 0.1;
const MOVE_JUMP_HEIGHT_PER_UNIT: f32 = 0.22;
const MOVE_JUMP_MAX_HEIGHT: f32 = 1.15;

#[derive(Clone, Debug)]
pub struct MoveJumpAnimation {
    base_transform: Transform,
    start_position: Vec3,
    end_position: Vec3,
    arc_height: f32,
    progress_ratio: f32,
}

impl MoveJumpAnimation {
    pub fn new(base_transform: Transform, end_position: Vec3) -> Self {
        let start_position = base_transform.get_position();
        let horizontal_distance = Vec3::new(
            end_position.x - start_position.x,
            0.0,
            end_position.z - start_position.z,
        )
        .length();
        let arc_height = (MOVE_JUMP_BASE_HEIGHT + horizontal_distance * MOVE_JUMP_HEIGHT_PER_UNIT)
            .min(MOVE_JUMP_MAX_HEIGHT);

        Self {
            base_transform,
            start_position,
            end_position,
            arc_height,
            progress_ratio: 0.0,
        }
    }

    fn current_position(&self) -> Vec3 {
        let t = self.progress_ratio;
        let base = self.start_position.lerp(self.end_position, t);
        let arc = 4.0 * self.arc_height * t * (1.0 - t);
        Vec3::new(base.x, base.y + arc, base.z)
    }
}

impl Animation for MoveJumpAnimation {
    fn progress(&mut self, delta_time: Duration) {
        let delta_progress = delta_time.as_secs_f32().min(0.05) / MOVE_JUMP_DURATION_SECONDS;
        self.progress_ratio = (self.progress_ratio + delta_progress).clamp(0.0, 1.0);
    }

    fn get_animation_transform(&self) -> Transform {
        let mut transform = self.base_transform;
        let position = if self.is_finished() {
            self.end_position
        } else {
            self.current_position()
        };
        transform.set_position(position);
        transform
    }

    fn is_finished(&self) -> bool {
        self.progress_ratio >= 1.0
    }

    fn blocks_input(&self) -> bool {
        true
    }
}
