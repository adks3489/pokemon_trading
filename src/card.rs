pub const NUM_CARDS: usize = 4;

pub fn is_valid(id: u32) -> bool {
    id < NUM_CARDS.try_into().unwrap()
}