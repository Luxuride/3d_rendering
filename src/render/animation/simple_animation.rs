use crate::render::animation::Animation;
use crate::render::buffers::transform::Transform;
use glam::Vec3;
use std::time::Duration;

const ANIMATION_DURATION: u128 = 10000;

#[derive(Copy, Clone, Default)]
pub struct SimpleAnimation {
    animation_time: u128,
}

impl SimpleAnimation {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { animation_time: 0 }
    }
}

impl Animation for SimpleAnimation {
    fn update_time(&mut self, delta_time: Duration) {
        self.animation_time += delta_time.as_millis();
        self.animation_time %= ANIMATION_DURATION;
    }

    fn get_animation_transform(&self, transform: &Transform) -> Transform {
        let mut transform = *transform;
        let pos = self.animation_time.min(10000 - (self.animation_time));
        transform.set_position(
            transform.get_position()
                + Vec3::new(
                    pos as f32 * 4.0 / 10000.0,
                    pos as f32 * 2.2 / 10000.0,
                    pos as f32 * 2.1 / 10000.0,
                ),
        );
        transform
    }
}
