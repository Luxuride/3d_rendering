pub mod chaos_gravity;
pub mod move_jump;

use std::time::Duration;

use crate::render::buffers::transform::Transform;

pub trait Animation {
    fn progress(&mut self, delta_time: Duration);
    fn get_animation_transform(&self) -> Transform;
    fn is_finished(&self) -> bool {
        false
    }
    fn blocks_input(&self) -> bool {
        false
    }
}
