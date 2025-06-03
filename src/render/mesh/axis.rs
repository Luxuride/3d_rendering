use crate::render::mesh::MeshBuilder;
use crate::render::model::vertex::vertex_raw::VertexRaw;
use cgmath::{Deg, Quaternion, Rotation3};

#[rustfmt::skip]
const VERTICES: [VertexRaw; 2] = [
    VertexRaw { position: [0.0, 0.0, 0.0], },
    VertexRaw { position: [1.0, 0.0, 0.0], },
];

#[rustfmt::skip]
const INDICES: [u16; 3] = [
    0, 1, 0,
];

fn axis_mesh_builder() -> MeshBuilder {
    MeshBuilder::default()
        .vertices(VERTICES.to_vec())
        .indices(INDICES.to_vec())
}

pub fn x_axis_mesh_builder() -> MeshBuilder {
    axis_mesh_builder()
}

pub fn y_axis_mesh_builder() -> MeshBuilder {
    axis_mesh_builder().rotation(Quaternion::from_angle_y(Deg(90.0)))
}

pub fn z_axis_mesh_builder() -> MeshBuilder {
    axis_mesh_builder().rotation(Quaternion::from_angle_z(Deg(90.0)))
}
