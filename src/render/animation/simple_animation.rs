use crate::render::animation::Animation;
use crate::render::buffers::transform::Transform;
use std::time::Duration;

const ANIMATION_DURATION: u128 = 10000;

#[derive(Copy, Clone, Default)]
pub struct SimpleAnimation {
    animation_time: u128,
}
impl Animation for SimpleAnimation {
    fn update_time(&mut self, delta_time: Duration) {
        self.animation_time += delta_time.as_millis();
        self.animation_time %= ANIMATION_DURATION;
    }

    fn get_animation_transform(&self, transform: &Transform) -> Transform {
        let mut transform = *transform;
        let pos = self.animation_time.min(10000 - (self.animation_time));
        transform.position.x += pos as f32 * 4.0 / 10000.0;
        transform.position.y += pos as f32 * 2.2 / 10000.0;
        transform.position.z += pos as f32 * 2.1 / 10000.0;
        transform
    }
}
