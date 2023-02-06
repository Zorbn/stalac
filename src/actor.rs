use crate::{chunk::Chunk, input::Input};

const GRAVITY: f32 = 30.0;
const JUMP_FORCE: f32 = 9.0;

pub struct Actor {
    speed: f32,
    size: cgmath::Vector3<f32>,
    position: cgmath::Vector3<f32>,
    look_x: f32,
    look_y: f32,
    y_velocity: f32,
    grounded: bool,
}

impl Actor {
    pub fn new(position: cgmath::Vector3<f32>, size: cgmath::Vector3<f32>, speed: f32) -> Self {
        Self {
            position,
            size,
            speed,
            look_x: 0.0,
            look_y: 0.0,
            y_velocity: 0.0,
            grounded: false,
        }
    }

    pub fn update(&mut self, _input: &mut Input, chunk: &Chunk, delta_time: f32) {
        self.grounded = chunk
            .get_block_collision(
                self.position - cgmath::Vector3::new(0.0, 0.1, 0.0),
                self.size,
            )
            .is_some();

        // If the player is moving towards the ground while touching it, snap to the floor
        // and prevent y_velocity from building up over time.
        if self.grounded && self.y_velocity < 0.0 {
            self.snap_to_floor()
        }

        self.apply_gravity(delta_time);

        if !self.step(
            cgmath::Vector3::unit_y(),
            self.y_velocity() * delta_time,
            chunk,
            false,
        ) {
            self.reset_y_velocity();
        }
    }

    pub fn step(
        &mut self,
        dir: cgmath::Vector3<f32>,
        speed: f32,
        chunk: &Chunk,
        no_clip: bool,
    ) -> bool {
        let old_position = self.position;

        let velocity = dir * speed;
        self.position += velocity;

        if !no_clip
            && chunk
                .get_block_collision(self.position, self.size)
                .is_some()
        {
            self.position = old_position;
            return false;
        }

        true
    }

    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.look_x += delta_x;
        self.look_y += delta_y;
        self.look_x = self.look_x.clamp(-89.0, 89.0);
    }

    pub fn teleport(&mut self, position: cgmath::Vector3<f32>) {
        self.position = position;
    }

    fn snap_to_floor(&mut self) {
        self.reset_y_velocity();
        self.position.y = self.position.y.floor() + self.size.y * 0.5;
    }

    fn reset_y_velocity(&mut self) {
        self.y_velocity = 0.0;
    }

    fn apply_gravity(&mut self, delta_time: f32) {
        self.y_velocity -= GRAVITY * delta_time;
    }

    pub fn jump(&mut self) {
        self.y_velocity = JUMP_FORCE;
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn size(&self) -> cgmath::Vector3<f32> {
        self.size
    }

    pub fn head_position(&self) -> cgmath::Vector3<f32> {
        self.position + cgmath::Vector3::new(0.0, self.size.y * 0.4, 0.0)
    }

    pub fn look_x(&self) -> f32 {
        self.look_x
    }

    pub fn look_y(&self) -> f32 {
        self.look_y
    }

    pub fn y_velocity(&self) -> f32 {
        self.y_velocity
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn grounded(&self) -> bool {
        self.grounded
    }
}
