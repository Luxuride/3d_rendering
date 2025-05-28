use cgmath::{Matrix4, Point3, Vector3, Deg, perspective, InnerSpace};

pub struct Camera {
    pub position: Point3<f32>,
    yaw: f32,   // Rotation around the global Y-axis (left/right)
    pitch: f32, // Rotation around the camera's local X-axis (up/down)
    fov_y: f32, // Field of View in degrees
    z_near: f32,
    z_far: f32,
    aspect_ratio: f32,
    sensitivity: f32,
    move_speed: f32,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        yaw: f32,
        pitch: f32,
        fov_y: f32,
        z_near: f32,
        z_far: f32,
        aspect_ratio: f32,
        move_speed: f32,
    ) -> Self {
        Self {
            position,
            yaw,
            pitch,
            fov_y,
            z_near,
            z_far,
            aspect_ratio,
            sensitivity: 0.1,
            move_speed,
        }
    }

    // Processes raw mouse delta to update yaw and pitch (orientation).
    pub fn process_mouse_movement(&mut self, mouse_delta_x: f32, mouse_delta_y: f32) {
        self.yaw += mouse_delta_x * self.sensitivity;
        // Pitch: Current implementation for "inverted Y-axis" (moving mouse down makes camera look down)
        self.pitch -= mouse_delta_y * self.sensitivity;

        self.pitch = self.pitch.clamp(-89.0, 89.0);
    }

    // Processes keyboard input for camera translation (position).
    pub fn process_keyboard_input(&mut self, direction: CameraMovement) {
        let forward = self.get_forward_vector();
        let right = self.get_right_vector();

        match direction {
            CameraMovement::Forward => self.position += forward * self.move_speed,
            CameraMovement::Backward => self.position -= forward * self.move_speed,
            CameraMovement::Left => self.position -= right * self.move_speed,
            CameraMovement::Right => self.position += right * self.move_speed,
            CameraMovement::Up => self.position += Vector3::unit_y() * self.move_speed, // Move along world Y
            CameraMovement::Down => self.position -= Vector3::unit_y() * self.move_speed, // Move along world Y
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
        ).normalize()
    }

    // Get the right vector of the camera (orientation).
    fn get_right_vector(&self) -> Vector3<f32> {
        self.get_forward_vector().cross(Vector3::unit_y()).normalize()
    }

    // Get the up vector of the camera (orientation).
    fn get_up_vector(&self) -> Vector3<f32> {
        self.get_right_vector().cross(self.get_forward_vector()).normalize()
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
}