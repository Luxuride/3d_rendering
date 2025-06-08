pub mod simple_animation;

use std::time::Duration;
use crate::render::buffers::transform::Transform;

pub trait Animation {
    fn update_time(&mut self, delta_time: Duration);
    fn get_animation_transform(&self, transform: &Transform) -> Transform;
}
