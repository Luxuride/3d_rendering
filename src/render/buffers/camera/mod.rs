use camera_raw::CameraRaw;
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3, perspective};
use std::time::Duration;

pub mod camera_raw;

pub struct CameraBuilder {
    position: Point3<f32>,
    yaw: f32,
    pitch: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
    aspect_ratio: f32,
    sensitivity: f32,
    move_speed: f32,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: 90.0,
            pitch: 0.0,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
            aspect_ratio: 1.0,
            sensitivity: 0.1,
            move_speed: 1.0,
        }
    }
}

impl CameraBuilder {
    #[allow(dead_code)]
    pub fn position(mut self, position: Point3<f32>) -> Self {
        self.position = position;
        self
    }
    #[allow(dead_code)]
    pub fn yaw(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self
    }
    #[allow(dead_code)]
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }
    #[allow(dead_code)]
    pub fn fov_y(mut self, fov_y: f32) -> Self {
        self.fov_y = fov_y;
        self
    }
    #[allow(dead_code)]
    pub fn z_near(mut self, z_near: f32) -> Self {
        self.z_near = z_near;
        self
    }
    #[allow(dead_code)]
    pub fn z_far(mut self, z_far: f32) -> Self {
        self.z_far = z_far;
        self
    }
    #[allow(dead_code)]
    pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }
    #[allow(dead_code)]
    pub fn sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }
    #[allow(dead_code)]
    pub fn move_speed(mut self, move_speed: f32) -> Self {
        self.move_speed = move_speed;
        self
    }
    #[allow(dead_code)]
    pub fn build(self) -> Camera {
        Camera {
            position: self.position,
            yaw: self.yaw,
            pitch: self.pitch,
            fov_y: self.fov_y,
            z_near: self.z_near,
            z_far: self.z_far,
            aspect_ratio: self.aspect_ratio,
            sensitivity: self.sensitivity,
            move_speed: self.move_speed,
        }
    }
}

pub struct Camera {
    position: Point3<f32>,
    yaw: f32,
    pitch: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
    aspect_ratio: f32,
    sensitivity: f32,
    move_speed: f32,
}

impl Camera {
    pub fn get_camera_uniform(&self) -> CameraRaw {
        CameraRaw::new(self.build_view_projection_matrix().into())
    }

    // Processes raw mouse delta to update yaw and pitch (orientation).
    pub fn process_mouse_movement(&mut self, mouse_delta_x: f32, mouse_delta_y: f32) {
        self.yaw += mouse_delta_x * self.sensitivity;
        // Pitch: Current implementation for "inverted Y-axis" (moving mouse down makes camera look down)
        self.pitch -= mouse_delta_y * self.sensitivity;

        self.pitch = self.pitch.clamp(-89.0, 89.0);
    }

    // Processes keyboard input for camera translation (position).
    pub fn process_keyboard_input(&mut self, direction: CameraMovement, delta_time: &Duration) {
        let forward = self.get_forward_vector();
        let right = self.get_right_vector();
        let up = self.get_up_vector();
        let speed_multiplier = self.move_speed * delta_time.as_secs_f32();
        match direction {
            CameraMovement::Forward => self.position += forward * speed_multiplier,
            CameraMovement::Backward => self.position -= forward * speed_multiplier,
            CameraMovement::Left => self.position -= right * speed_multiplier,
            CameraMovement::Right => self.position += right * speed_multiplier,
            CameraMovement::Up => self.position += up * speed_multiplier,
            CameraMovement::Down => self.position -= up * speed_multiplier,
            CameraMovement::FovUp => {
                self.fov_y = (self.fov_y + speed_multiplier * 10.0).clamp(10.0, 80.0)
            }
            CameraMovement::FovDown => {
                self.fov_y = (self.fov_y - speed_multiplier * 10.0).clamp(10.0, 80.0)
            }
        }
    }

    // Get the forward vector of the camera (orientation).
    fn get_forward_vector(&self) -> Vector3<f32> {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        Vector3::new(
            pitch_rad.cos() * yaw_rad.cos(),
            pitch_rad.sin(),
            pitch_rad.cos() * yaw_rad.sin(),
        )
        .normalize()
    }

    // Get the right vector of the camera (orientation).
    fn get_right_vector(&self) -> Vector3<f32> {
        self.get_forward_vector()
            .cross(Vector3::unit_y())
            .normalize()
    }

    // Get the up vector of the camera (orientation).
    fn get_up_vector(&self) -> Vector3<f32> {
        self.get_right_vector()
            .cross(self.get_forward_vector())
            .normalize()
    }

    // Build the view matrix (transforms world coordinates to camera coordinates).
    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        let forward = self.get_forward_vector();
        let target = self.position + forward;
        Matrix4::look_at_rh(self.position, target, self.get_up_vector())
    }

    // Build the projection matrix (transforms camera coordinates to clip space).
    pub fn build_projection_matrix(&self) -> Matrix4<f32> {
        perspective(Deg(self.fov_y), self.aspect_ratio, self.z_near, self.z_far)
    }

    // Build the combined view-projection matrix.
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        self.build_projection_matrix() * self.build_view_matrix()
    }

    // Updates the camera's aspect ratio.
    pub fn update_aspect_ratio(&mut self, new_aspect_ratio: f32) {
        self.aspect_ratio = new_aspect_ratio;
    }
    pub fn get_position(&self) -> &Point3<f32> {
        &self.position
    }
    pub fn get_fov(&self) -> f32 {
        self.fov_y
    }
    pub fn get_mov_speed_raw(&mut self) -> &mut f32 {
        &mut self.move_speed
    }
}

// Enum to define possible camera movement directions from keyboard input
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
    FovUp,
    FovDown,
}
