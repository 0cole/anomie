use rand::{Rng, rngs::SmallRng};

pub fn mutate_bytes(rng: &mut SmallRng, bytes: &mut [u8]) -> String {
    let mut mutation_desc = String::new();
    let index = rng.random_range(0..bytes.len());
    match rng.random_range(0..4) {
        0 => {
            // bitmask mutation
            let mask: u8 = rng.random();
            bytes[index] ^= mask;
            mutation_desc = format!("applying a bitmask at index {index}");
        }
        1 => {
            // bit flip
            let bit_index = rng.random_range(0..8);
            let mutated_byte = bytes[index] ^ (1 << bit_index);
            bytes[index] = mutated_byte;
            mutation_desc = format!("applying a bitflip at index {index}");
        }
        2 => {
            // byte insertion
            let new_byte: u8 = rng.random();
            bytes[index..].rotate_right(1);
            bytes[index] = new_byte;
            mutation_desc = format!("inserting the byte <{new_byte}> at index {index}");
        }
        3 => {
            // byte shift
            bytes.rotate_left(1);
            mutation_desc = "shifting everything left by 1".to_string();
        }
        _ => {}
    }
    mutation_desc
}
