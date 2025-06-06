use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use crate::render::model::mesh::MeshBuilder;

#[rustfmt::skip]
const VERTICES: [VertexRaw; 2] = [
    VertexRaw { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0]},
    VertexRaw { position: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0]},
];

#[rustfmt::skip]
const INDICES: [u32; 3] = [
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
    axis_mesh_builder()
}
pub fn z_axis_mesh_builder() -> MeshBuilder {
    axis_mesh_builder()
}
