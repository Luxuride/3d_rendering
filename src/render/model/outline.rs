use crate::render::model::Model;

#[allow(dead_code)]
#[derive(Default)]
pub struct Outline {
    model: Option<Model>,
}

impl Outline {
    // Getter method
    pub fn get_model(&self) -> &Option<Model> {
        &self.model
    }

    // Setter method
    pub fn set_model(&mut self, model: Option<Model>) {
        self.model = model;
    }
}
