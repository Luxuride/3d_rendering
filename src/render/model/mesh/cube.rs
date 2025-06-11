use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use crate::render::model::mesh::MeshBuilder;
use std::sync::LazyLock;

static VERTICES: LazyLock<[VertexRaw; 8]> = LazyLock::new(|| {
    [
        // Ignore normals by setting them to 0
        // Ignore texture coords by setting them to 0
        VertexRaw::new([-0.5, -0.5, 0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([0.5, -0.5, 0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([-0.5, 0.5, 0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([0.5, 0.5, 0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([-0.5, -0.5, -0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([0.5, -0.5, -0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([-0.5, 0.5, -0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
        VertexRaw::new([0.5, 0.5, -0.5], [0.0, 0.0], [0.0, 0.0, 0.0]),
    ]
});
#[rustfmt::skip]
const INDICES: [u32; 36] = [
    0, 1, 2,
    1, 3, 2,

    4, 6, 5,
    5, 6, 7,

    1, 5, 3,
    5, 7, 3,

    0, 2, 4,
    2, 6, 4,

    2, 3, 6,
    3, 7, 6,

    0, 4, 1,
    1, 4, 5,
];

pub fn cube_mesh_builder() -> MeshBuilder {
    MeshBuilder::default()
        .vertices(VERTICES.to_vec())
        .indices(INDICES.to_vec())
}
