pub const NUM_CARDS: usize = 4;

pub fn is_valid(id: i32) -> bool {
    id >= 0 && id < NUM_CARDS.try_into().unwrap()
}
