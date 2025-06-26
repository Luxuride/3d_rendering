use crate::render::buffers::transform::transform_raw::TransformRaw;
use glam::{Mat4, Quat, Vec3};

pub mod transform_raw;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub(crate) fn to_raw(self) -> TransformRaw {
        let transform = Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale);
        TransformRaw::new(transform.to_cols_array_2d())
    }
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }
}
