pub struct Health {
    amount: i32,
}

impl Health {
    pub fn new(amount: i32) -> Self {
        Self { amount }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.amount -= amount;

        println!("health: {}", self.amount);
    }
}
