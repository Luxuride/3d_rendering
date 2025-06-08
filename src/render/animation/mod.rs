pub mod simple_animation;

use crate::render::buffers::transform::Transform;
use std::time::Duration;

pub trait Animation {
    fn update_time(&mut self, delta_time: Duration);
    fn get_animation_transform(&self, transform: &Transform) -> Transform;
}
