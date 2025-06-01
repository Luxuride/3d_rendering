use crate::render::model::transformation::transformation_raw::TransformationRaw;

pub mod transformation_raw;

pub struct Transformation {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Transformation {
    pub(crate) fn to_raw(&self) -> TransformationRaw {
        TransformationRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
                .into(),
        }
    }
}
