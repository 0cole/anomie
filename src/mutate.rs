use anyhow::Result;
use log::debug;
use rand::{Rng, random, rng, rngs::SmallRng};
use std::{collections::HashSet, fs, path::PathBuf};

use crate::fuzzers::jpeg::JpegObject;

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

#[allow(clippy::too_many_lines)]
pub fn mutate_jpeg(rng: &mut SmallRng, file: &PathBuf) -> Result<()> {
    // let img = ImageReader::open(file)?.decode().unwrap();
    let jpg = JpegObject::new(file)?;
    let bytes: Vec<u8> = fs::read(file)?;
    let mutated_file_name = "temp/mutated.jpg";

    let total_mutations = rng.random_range(1..=2);
    for _ in 0..total_mutations {
        let mut mutated = bytes.clone();
        let mut mutated_jpg: JpegObject = jpg.clone();
        match rng.random_range(0..=9) {
            0 => {
                // [RAW BYTES] truncate the middle
                debug!("truncating {} at its midpoint", file.display());
                fs::write(mutated_file_name, &bytes[..bytes.len() / 2])?;
            }
            1 => {
                // remove EOI - last 2 bytes are a flag that represent the end
                // of the jpeg
                if let Some(eoi) = mutated_jpg.eoi_mut() {
                    debug!("removing the EOI of {}", file.display());
                    eoi.clear();
                    mutated_jpg.write_to_file(mutated_file_name)?;
                }
            }
            2 => {
                // corrupt SOI - replace the traditional jpeg start flag with a random byte
                if let Some(soi) = mutated_jpg.soi_mut() {
                    debug!("corrupting the SOI of {}", file.display());
                    let rand_byte = rng.random::<u8>();
                    soi[1] = rand_byte;
                    mutated_jpg.write_to_file(mutated_file_name)?;
                }
            }
            3 => {
                // corrupt SOF - change the expected width/height of the file
                // xFF xC0 corresponds to baseline
                // xFF xC2 corresponds to progressive
                if let Some(sof) = mutated_jpg.sof_mut() {
                    debug!(
                        "overwriting the expected width/height of {}",
                        file.display()
                    );
                    sof[5] = random::<u8>();
                    sof[6] = random::<u8>();
                    sof[7] = random::<u8>();
                    sof[8] = random::<u8>();
                    mutated_jpg.write_to_file(mutated_file_name)?;
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
                debug!(
                    "byteflipping {:.2}% of {}",
                    mutation_rate * 100.0,
                    file.display()
                );
                fs::write(mutated_file_name, mutated)?;
            }
            5 => {
                // add trailing garbage bytes at end
                if let Some(eoi) = mutated_jpg.eoi_mut() {
                    let tail_length = rng.random_range(0..10_000);
                    debug!(
                        "adding {tail_length} bytes at the end of {}",
                        file.display()
                    );

                    for _ in 0..tail_length {
                        eoi.push(rng.random::<u8>());
                    }
                }
            }
            6 => {
                // overwrite segment lengths, the two bytes after the segment header indicate the segment length
                let (segment, segment_name) = match rng.random_range(0..=4) {
                    0 => (mutated_jpg.app_mut(), "APP"),
                    1 => (mutated_jpg.random_dqt_mut(rng), "DQT"),
                    2 => (mutated_jpg.sof_mut(), "SOF"),
                    3 => (mutated_jpg.random_dht_mut(rng), "DHT"),
                    4 => (mutated_jpg.sos_mut(), "SOS"),
                    _ => unreachable!(),
                };

                if let Some(data) = segment {
                    // generate two bytes from the same rng call
                    let [high_byte, low_byte] = rng.random::<u16>().to_be_bytes();
                    data[2] = high_byte;
                    data[3] = low_byte;
                    debug!(
                        "overwriting the segment length of the {segment_name} segment to be {:?} for {}",
                        (u16::from(high_byte) >> 8) + u16::from(low_byte),
                        file.display()
                    );
                    mutated_jpg.write_to_file(mutated_file_name)?;
                }
            }
            7 => {
                // rearrange the segments
                let mut first = usize::MAX;
                let mut second = usize::MAX;
                while first == second {
                    first = rng.random_range(0..mutated_jpg.segments.len());
                    second = rng.random_range(0..mutated_jpg.segments.len());
                }
                let temp = mutated_jpg.segments[first].clone();
                mutated_jpg.segments[first] = mutated_jpg.segments[second].clone();
                mutated_jpg.segments[second] = temp;
                debug!(
                    "swapping the two segments at positions {first} and {second} of {}",
                    file.display()
                );

                mutated_jpg.write_to_file(mutated_file_name)?;
            }
            8 => {
                // alter dht tables
                if let Some(dht) = mutated_jpg.random_dht_mut(rng) {
                    mutate_bytes(&mut dht[5..]);
                    debug!("mutating one of the dhts of {}", file.display());
                    mutated_jpg.write_to_file(mutated_file_name)?;
                }
            }
            9 => {
                // alter dqt tables
                if let Some(dqt) = mutated_jpg.random_dqt_mut(rng) {
                    mutate_bytes(&mut dqt[5..]);
                    debug!("mutating one of the dqts of {}", file.display());
                    mutated_jpg.write_to_file(mutated_file_name)?;
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
