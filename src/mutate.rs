use std::{collections::HashSet, fs, io, path::PathBuf};

use image::ImageReader;
use log::{debug, info, warn};
use rand::{Rng, random, rng, rngs::ThreadRng};

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

pub fn mutate_jpeg(mut rng: ThreadRng, file: &PathBuf) -> io::Result<()> {
    let img = ImageReader::open(file)?.decode().unwrap();
    let bytes: Vec<u8> = fs::read(file)?;
    let mutated_file_name = "mutated.jpg";

    let total_mutations = rng.random_range(0..2);
    for _ in 0..total_mutations {
        let mut mutated = bytes.clone();
        match rng.random_range(0..5) {
            0 => {
                // truncate the middle
                fs::write(mutated_file_name, &bytes[..bytes.len() / 2])?;
                debug!("truncating {} at its midpoint", file.display());
            }
            1 => {
                // remove EOF - last 2 bytes are a flag that represent the end
                // of the jpeg
                fs::write(mutated_file_name, &bytes[..bytes.len() - 2])?;
                debug!("removing the EOF of {}", file.display());
            }
            2 => {
                // corrupt SOI - replace the traditional jpeg start flag with a random byte
                let rand_byte = rng.random::<u8>();
                mutated[1] = rand_byte;
                fs::write(mutated_file_name, &mutated)?;
                debug!("corrupted the SOI of {}", file.display());
            }
            3 => {
                // corrupt SOF - change the expected width/height of the file
                // xFF xC0 corresponds to baseline
                // xFF xC2 corresponds to progressive
                if let Some(sof_start_index) = bytes
                    .windows(2)
                    .position(|w| w == [0xFF, 0xC0] || w == [0xFF, 0xC2])
                {
                    mutated[sof_start_index + 5] = random::<u8>();
                    mutated[sof_start_index + 6] = random::<u8>();
                    mutated[sof_start_index + 7] = random::<u8>();
                    mutated[sof_start_index + 8] = random::<u8>();
                    fs::write(mutated_file_name, &mutated)?;
                    debug!("overwrote the expected width/height of {}", file.display());
                } else {
                    warn!("jpg does not contain a SOF");
                }
            }
            4 => {
                // byteflip non-header data, flip at most 2% of all non-header bytes
                let mutation_rate = rng.random_range(0.001..0.02);
                let total_byteflip_mutations = (bytes.len() as f64 * mutation_rate).ceil() as u64;

                // first collect all header indicies
                let mut header_indicies: HashSet<usize> = HashSet::new();
                for i in 0..bytes.len() - 1 {
                    // in the image data, xFF bytes are always followed by x00
                    if bytes[i] == 0xFF && bytes[i + 1] != 0x00 {
                        header_indicies.insert(i);
                        header_indicies.insert(i + 1);
                    }
                }

                for _ in 0..total_byteflip_mutations {
                    // skip last 2 bytes of file
                    let mut index = rng.random_range(0..bytes.len() - 3);
                    while header_indicies.contains(&index) {
                        index += 1;
                    }
                    let rand_byte = rng.random::<u8>();
                    mutated[index] = rand_byte;
                }
                fs::write(mutated_file_name, mutated)?;
                debug!(
                    "byteflipped {:.2}% of {}",
                    mutation_rate * 100.0,
                    file.display()
                );
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
