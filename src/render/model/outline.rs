use crate::render::model::mesh::Mesh;
use crate::render::model::Model;
use eframe::wgpu::{Device, Queue};

#[derive(Default)]
pub struct Outline {
    pub model: Option<Model>,
}

impl Outline {}
