use super::vertex::Vertex;

pub const CUBE_VERTICES: [[Vertex; 4]; 6] = [
    [
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            tex_index: 0,
        },
    ],
    [
        Vertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [1.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            tex_index: 0,
        },
    ],
    [
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            tex_index: 0,
        },
    ],
    [
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [1.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tex_index: 0,
        },
    ],
    [
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [1.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 1.0],
            tex_index: 0,
        },
    ],
    [
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            tex_index: 0,
        },
        Vertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            tex_index: 0,
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            tex_index: 0,
        },
    ],
];

pub const CUBE_INDICES: [[u32; 6]; 6] = [
    [0, 1, 2, 0, 2, 3],
    [0, 2, 1, 0, 3, 2],
    [0, 2, 1, 0, 3, 2],
    [0, 1, 2, 0, 2, 3],
    [0, 1, 2, 0, 2, 3],
    [0, 2, 1, 0, 3, 2],
];
