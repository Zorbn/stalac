pub struct Health {

}

impl Health {
    pub fn take_damage(&mut self, amount: i32) {
        println!("damaged: {}", amount);
    }
}