use std::collections::HashMap;

use super::{instance::Instance, sprite_mesh::UI_SPRITE_WIDTH};

use cgmath::prelude::*;

const CHARACTERS: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ ";

pub struct Gui {
    characters: HashMap<char, u32>,
    glyph_instances: Vec<Instance>,
}

impl Gui {
    pub fn new() -> Self {
        let mut characters = HashMap::new();
        CHARACTERS.chars().enumerate().for_each(|(i, char)| {
            characters.insert(char, i as u32);
        });

        Self {
            characters,
            glyph_instances: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.glyph_instances.clear();
    }

    pub fn write(&mut self, text: &str) {
        for (i, char) in text.chars().enumerate() {
            if let Some(char_index) = self.characters.get(&char) {
                self.glyph_instances.push(Instance {
                    position: cgmath::Vector3::new(i as f32 * UI_SPRITE_WIDTH, 0.0, 0.0),
                    rotation: cgmath::Quaternion::zero(),
                    tex_index: *char_index,
                })
            }
        }
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.glyph_instances
    }
}
