use super::vertex::Vertex;

pub const SPRITE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
        tex_index: 0,
    },
    Vertex {
        position: [0.5, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
        tex_index: 0,
    },
    Vertex {
        position: [0.5, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
        tex_index: 0,
    },
    Vertex {
        position: [-0.5, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
        tex_index: 0,
    },
];

pub const UI_SPRITE_WIDTH: f32 = 6.0 / 14.0;

pub const UI_SPRITE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
        tex_index: 0,
    },
    Vertex {
        position: [UI_SPRITE_WIDTH, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
        tex_index: 0,
    },
    Vertex {
        position: [UI_SPRITE_WIDTH, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
        tex_index: 0,
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
        tex_index: 0,
    },
];

pub const SPRITE_INDICES: &[u32] = &[0, 2, 1, 0, 1, 3];
