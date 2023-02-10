pub struct Fighter {
    attack_cooldown: f32,
}

impl Fighter {
    pub fn new() -> Self {
        Self {
            attack_cooldown: 1.0,
        }
    }

    pub fn get_attack(&mut self) -> i32 {
        self.attack_cooldown = 0.0;
        10
    }
}