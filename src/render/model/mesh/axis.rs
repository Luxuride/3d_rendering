use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use crate::render::model::mesh::MeshBuilder;
use std::sync::LazyLock;

static VERTICES: LazyLock<[VertexRaw; 2]> = LazyLock::new(|| {
    [
        VertexRaw::new([0.0, 0.0, 0.0], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([1.0, 0.0, 0.0], [0.0, 0.0], [0.0, 0.0, 0.0]),
    ]
});
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
