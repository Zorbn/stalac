use std::collections::HashMap;

use super::{instance::Instance, sprite_mesh::UI_SPRITE_WIDTH};

use cgmath::prelude::*;

const CHARACTERS: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ ";

pub struct Gui {
    characters: HashMap<char, u32>,
    glyph_instances: Vec<Instance>,
    write_line: f32,
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
            write_line: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.glyph_instances.clear();
        self.write_line = 0.0;
    }

    pub fn write(&mut self, text: &str) -> f32 {
        let write_line = self.write_line;
        self.write_line += 1.0;

        for (i, char) in text.chars().enumerate() {
            if let Some(char_index) = self.characters.get(&char) {
                self.glyph_instances.push(Instance {
                    position: cgmath::vec3(i as f32 * UI_SPRITE_WIDTH, write_line, 0.0),
                    rotation: cgmath::Quaternion::zero(),
                    tex_index: *char_index,
                })
            }
        }

        write_line
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.glyph_instances
    }
}
