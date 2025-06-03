use crate::render::model::transform::transform_raw::TransformRaw;

pub mod transform_raw;

pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Transform {
    pub(crate) fn to_raw(&self) -> TransformRaw {
        let transform = cgmath::Matrix4::from_translation(self.position)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        TransformRaw::new(transform.into())
    }
}
