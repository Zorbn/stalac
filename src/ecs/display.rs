use super::ecs::System;

pub struct Display {
    tex_index: u32,
}

impl Display {
    pub fn new(tex_index: u32) -> Self {
        Self { tex_index }
    }

    pub fn tex_index(&self) -> u32 {
        self.tex_index
    }
}
