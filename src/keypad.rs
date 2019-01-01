pub enum Key {
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    KA,
    KB,
    KC,
    KD,
    KE,
    KF,
    Unknown,
}

pub trait Keypad {
    fn key_to_u8(&self, k: &Key) -> u8;
    fn key_to_index(&self, k: &Key) -> Option<usize>;
    fn key_from_u8(&self, k: &u8) -> Key;
    fn key_pressed(&self, key: &Key) -> bool;
    fn any_key_pressed(&self) -> Option<Key>;
}
