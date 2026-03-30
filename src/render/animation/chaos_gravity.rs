use crate::render::animation::Animation;
use crate::render::buffers::transform::Transform;
use glam::{EulerRot, Quat, Vec2, Vec3};
use std::time::Duration;

const CAPTURE_CHAOS_GRAVITY: f32 = -19.6;
const CAPTURE_CHAOS_MIN_UPWARD_SPEED: f32 = 4.0;
const CAPTURE_CHAOS_MAX_UPWARD_SPEED: f32 = 9.0;
const CAPTURE_CHAOS_MIN_HORIZONTAL_SPEED: f32 = 2.0;
const CAPTURE_CHAOS_MAX_HORIZONTAL_SPEED: f32 = 7.0;
const CAPTURE_CHAOS_MIN_ANGULAR_SPEED: f32 = -12.0;
const CAPTURE_CHAOS_MAX_ANGULAR_SPEED: f32 = 12.0;
const CAPTURE_CHAOS_LIFETIME_SECONDS: f32 = 4.0;
const CAPTURE_CHAOS_MIN_Y: f32 = -120.0;
const CAPTURE_CHAOS_MAX_HORIZONTAL_DISTANCE: f32 = 120.0;

#[derive(Clone, Debug)]
pub struct ChaosGravityAnimation {
    initial_position: Vec3,
    velocity: Vec3,
    angular_velocity: Vec3,
    elapsed_seconds: f32,
}

impl ChaosGravityAnimation {
    pub fn new(initial_position: Vec3, seed: u32) -> Self {
        let heading = random_range(seed.wrapping_add(11), 0.0, std::f32::consts::TAU);
        let horizontal_speed = random_range(
            seed.wrapping_add(23),
            CAPTURE_CHAOS_MIN_HORIZONTAL_SPEED,
            CAPTURE_CHAOS_MAX_HORIZONTAL_SPEED,
        );
        let upward_speed = random_range(
            seed.wrapping_add(37),
            CAPTURE_CHAOS_MIN_UPWARD_SPEED,
            CAPTURE_CHAOS_MAX_UPWARD_SPEED,
        );

        let velocity = Vec3::new(
            heading.cos() * horizontal_speed,
            upward_speed,
            heading.sin() * horizontal_speed,
        );

        let angular_velocity = Vec3::new(
            random_range(
                seed.wrapping_add(41),
                CAPTURE_CHAOS_MIN_ANGULAR_SPEED,
                CAPTURE_CHAOS_MAX_ANGULAR_SPEED,
            ),
            random_range(
                seed.wrapping_add(59),
                CAPTURE_CHAOS_MIN_ANGULAR_SPEED,
                CAPTURE_CHAOS_MAX_ANGULAR_SPEED,
            ),
            random_range(
                seed.wrapping_add(73),
                CAPTURE_CHAOS_MIN_ANGULAR_SPEED,
                CAPTURE_CHAOS_MAX_ANGULAR_SPEED,
            ),
        );

        Self {
            initial_position,
            velocity,
            angular_velocity,
            elapsed_seconds: 0.0,
        }
    }

    fn current_displacement(&self) -> Vec3 {
        let t = self.elapsed_seconds;
        Vec3::new(
            self.velocity.x * t,
            self.velocity.y * t + 0.5 * CAPTURE_CHAOS_GRAVITY * t * t,
            self.velocity.z * t,
        )
    }
}

impl Animation for ChaosGravityAnimation {
    fn progress(&mut self, delta_time: Duration) {
        self.elapsed_seconds += delta_time.as_secs_f32().min(0.05);
    }

    fn get_animation_transform(&self, transform: &Transform) -> Transform {
        let mut transform = *transform;
        let displacement = self.current_displacement();
        transform.set_position(self.initial_position + displacement);

        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            self.angular_velocity.x * self.elapsed_seconds,
            self.angular_velocity.y * self.elapsed_seconds,
            self.angular_velocity.z * self.elapsed_seconds,
        );
        transform.rotation(rotation)
    }

    fn is_finished(&self) -> bool {
        if self.elapsed_seconds >= CAPTURE_CHAOS_LIFETIME_SECONDS {
            return true;
        }

        let displacement = self.current_displacement();
        let position = self.initial_position + displacement;
        let horizontal = Vec2::new(displacement.x, displacement.z).length();

        position.y < CAPTURE_CHAOS_MIN_Y || horizontal > CAPTURE_CHAOS_MAX_HORIZONTAL_DISTANCE
    }
}

fn random_range(seed: u32, min: f32, max: f32) -> f32 {
    let t = random_unit(seed);
    min + (max - min) * t
}

fn random_unit(seed: u32) -> f32 {
    let x = (seed as f32 * 12.989_8 + 78.233).sin() * 43_758.547;
    x.fract().abs()
}
