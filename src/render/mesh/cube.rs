use crate::render::mesh::MeshBuilder;
use crate::render::model::vertex::vertex_raw::VertexRaw;

const VERTICES: [VertexRaw; 8] = [
    VertexRaw {
        position: [-1.0, -1.0, 1.0],
    },
    VertexRaw {
        position: [1.0, -1.0, 1.0],
    },
    VertexRaw {
        position: [-1.0, 1.0, 1.0],
    },
    VertexRaw {
        position: [1.0, 1.0, 1.0],
    },
    VertexRaw {
        position: [-1.0, -1.0, -1.0],
    },
    VertexRaw {
        position: [1.0, -1.0, -1.0],
    },
    VertexRaw {
        position: [-1.0, 1.0, -1.0],
    },
    VertexRaw {
        position: [1.0, 1.0, -1.0],
    },
];
const INDICES: [u16; 36] = [
    0, 1, 2, 1, 3, 2, 4, 6, 5, 5, 6, 7, 1, 5, 3, 5, 7, 3, 0, 2, 4, 2, 6, 4, 2, 3, 6, 3, 7, 6, 0, 4,
    1, 1, 4, 5,
];

pub fn cube_mesh_builder() -> MeshBuilder {
    MeshBuilder::default()
        .vertices(VERTICES.to_vec())
        .indices(INDICES.to_vec())
}
