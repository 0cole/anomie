use rand::{Rng, rng};

pub fn mutate_string(s: &str) -> String {
    let mut rng = rng();

    let mut m = s.to_string();

    for _ in 0..10 {
        // the mutation is ASCII if 0, otherwise unicode if 1
        let mutation: char = match rng.random_range(0..1) {
            0 => u8::try_from(rng.random_range(0..=255)).unwrap() as char,
            1 => {
                let unicode_int = rng.random_range(0..=2_u32.pow(16));
                std::char::from_u32(unicode_int).unwrap()
            }
            _ => unreachable!(),
        };

        match rng.random_range(0..4) {
            0 => m.push(mutation),                          // Append a byte
            1 => m = m.replace(' ', &mutation.to_string()), // Replace
            2 => {
                let mut chars: Vec<char> = m.chars().collect(); // Insert a random ASCII byte
                let pos = rng.random_range(0..=chars.len());
                chars.insert(pos, mutation);
                m = chars.iter().collect();
            }
            3 => {
                let mut chars: Vec<char> = m.chars().collect(); // Insert between 1-100 null bytes
                let pos = rng.random_range(0..=chars.len());
                for _ in 0..rng.random_range(1..100) {
                    chars.insert(pos, '\0');
                }
                m = chars.iter().collect();
            }
            // 3 => m = m.chars().rev().collect(),         // Reverse
            _ => {}
        }
    }
    m
}

pub fn mutate_bytes(bytes: &mut [u8]) {
    let mut rng = rng();

    for _ in 0..10 {
        let index = rng.random_range(0..bytes.len());
        match rng.random_range(0..4) {
            0 => {
                // bitmask mutation
                let mask: u8 = rng.random();
                bytes[index] ^= mask;
            }
            1 => {
                // bit flip
                let bit_index = rng.random_range(0..8);
                let mutated_byte = bytes[index] ^ (1 << bit_index);
                bytes[index] = mutated_byte;
            }
            2 => {
                // byte insertion
                let new_byte: u8 = rng.random();
                bytes[index..].rotate_right(1);
                bytes[index] = new_byte;
            }
            3 => {
                // byte shift
                bytes.rotate_left(1);
            }
            _ => {}
        }
    }
}
