#[derive(Copy, Clone)]
pub enum Direction {
    Forward = 0,
    Backward = 1,
    Right = 2,
    Left = 3,
    Up = 4,
    Down = 5,
}

pub fn index_to_dir(i: usize) -> Direction {
    match i {
        0 => Direction::Forward,
        1 => Direction::Backward,
        2 => Direction::Right,
        3 => Direction::Left,
        4 => Direction::Up,
        5 => Direction::Down,
        _ => panic!("Failed to convert index to direction!")
    }
}

pub fn dir_to_offset(dir: Direction) -> (i32, i32, i32) {
    match dir {
        Direction::Up => (0, 1, 0),
        Direction::Down => (0, -1, 0),
        Direction::Forward => (0, 0, -1),
        Direction::Backward => (0, 0, 1),
        Direction::Left => (-1, 0, 0),
        Direction::Right => (1, 0, 0),
    }
}