use glam::Vec3;

use crate::game_logic::chess::Square;

pub fn square_to_world(square: Square, board_min: Vec3, board_max: Vec3) -> Vec3 {
    let square_width = (board_max.x - board_min.x) / 8.0;
    let square_depth = (board_max.z - board_min.z) / 8.0;
    let x = board_min.x + (square.file() as f32 + 0.5) * square_width;
    let z = board_min.z + (square.rank() as f32 + 0.5) * square_depth;
    Vec3::new(x, board_max.y + 0.01, z)
}

pub fn world_to_square(point: Vec3, board_min: Vec3, board_max: Vec3) -> Option<Square> {
    if point.x < board_min.x
        || point.x > board_max.x
        || point.z < board_min.z
        || point.z > board_max.z
    {
        return None;
    }

    let width = board_max.x - board_min.x;
    let depth = board_max.z - board_min.z;
    if width <= 0.0 || depth <= 0.0 {
        return None;
    }

    let rel_x = ((point.x - board_min.x) / width).clamp(0.0, 0.999_999);
    let rel_z = ((point.z - board_min.z) / depth).clamp(0.0, 0.999_999);

    let file = (rel_x * 8.0).floor() as u8;
    let rank = (rel_z * 8.0).floor() as u8;
    Square::new(file, rank)
}
