pub mod chaos_gravity;

use std::time::Duration;

use crate::render::buffers::transform::Transform;

pub trait Animation {
    fn progress(&mut self, delta_time: Duration);
    fn get_animation_transform(&self, transform: &Transform) -> Transform;
    fn is_finished(&self) -> bool {
        false
    }
}
