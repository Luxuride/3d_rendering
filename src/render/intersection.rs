use crate::render::buffers::camera::Camera;
use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    pub fn from_vertices(
        vertices: &[VertexRaw],
        indices: &[u32],
        triangle_index: usize,
    ) -> Option<Self> {
        let base_index = triangle_index * 3;
        if base_index + 2 >= indices.len() {
            return None;
        }

        let i0 = indices[base_index] as usize;
        let i1 = indices[base_index + 1] as usize;
        let i2 = indices[base_index + 2] as usize;

        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            return None;
        }

        let a = Vec3::from_array(vertices[i0].position());
        let b = Vec3::from_array(vertices[i1].position());
        let c = Vec3::from_array(vertices[i2].position());

        Some(Self::new(a, b, c))
    }
    pub fn get_a(&self) -> Vec3 {
        self.a
    }
    pub fn get_b(&self) -> Vec3 {
        self.b
    }
    pub fn get_c(&self) -> Vec3 {
        self.c
    }
}

/// Möller–Trumbore intersection algorithm
///
/// https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
pub fn moller_trumbore_intersection(
    origin: Vec3,
    direction: Vec3,
    triangle: Triangle,
) -> Option<Vec3> {
    let e1 = triangle.get_b() - triangle.get_a();
    let e2 = triangle.get_c() - triangle.get_a();

    let ray_cross_e2 = direction.cross(e2);
    let det = e1.dot(ray_cross_e2);

    if det > -f32::EPSILON && det < f32::EPSILON {
        return None; // This ray is parallel to this triangle.
    }

    let inv_det = 1.0 / det;
    let s = origin - triangle.get_a();
    let u = inv_det * s.dot(ray_cross_e2);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let s_cross_e1 = s.cross(e1);
    let v = inv_det * direction.dot(s_cross_e1);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = inv_det * e2.dot(s_cross_e1);

    if t > f32::EPSILON {
        // ray intersection
        let intersection_point = origin + direction * t;
        Some(intersection_point)
    } else {
        // This means that there is a line intersection but not a ray intersection.
        None
    }
}

pub fn screen_to_world_ray(screen_pos: Vec2, viewport_size: Vec2, camera: &Camera) -> Vec3 {
    let ndc_x = (2.0 * screen_pos.x) / viewport_size.x - 1.0;
    let ndc_y = 1.0 - (2.0 * screen_pos.y) / viewport_size.y;

    let ray_clip = Vec3::new(ndc_x, ndc_y, -1.0);

    let ray_eye = camera.build_projection_matrix().inverse()
        * Vec3::new(ray_clip.x, ray_clip.y, -1.0).extend(0.0);
    let ray_eye = Vec3::new(ray_eye.x, ray_eye.y, -1.0);

    let ray_world = camera.build_view_matrix().inverse() * ray_eye.extend(0.0);
    let ray_world = Vec3::new(ray_world.x, ray_world.y, ray_world.z).normalize();

    if !ray_world.is_finite() {
        return Vec3::new(0.0, 0.0, 1.0);
    }

    ray_world
}
