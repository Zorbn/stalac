pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        assert_ne!(seed, 0);

        Self {
            state: seed,
        }
    }

    pub fn range(&mut self, max: u32) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 5;
        self.state % max
    }
}