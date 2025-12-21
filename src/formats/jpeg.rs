use super::template::FileFormat;
use crate::mutate::mutate_bytes;
use anyhow::Result;
use rand::{Rng, rngs::SmallRng};
use std::{collections::HashSet, path::Path};

pub struct Jpeg;
#[derive(Clone)]
pub struct JpegModel {
    pub soi: Vec<u8>,
    pub segments: Vec<JpegSegment>,
    pub eoi: Vec<u8>,
}

#[derive(Clone)]
pub enum JpegSegment {
    App(Vec<u8>),
    Dqt(Vec<u8>),
    Sof(Vec<u8>),
    Dht(Vec<u8>),
    Sos(Vec<u8>),
    Dat(Vec<u8>),
}

impl FileFormat for Jpeg {
    type Model = JpegModel;
    const EXT: &str = "jpg";

    #[allow(clippy::too_many_lines, unused_assignments)]
    fn parse(input: &[u8]) -> Result<Self::Model> {
        let mut segments = Vec::new();
        let mut i = 0;
        let len = input.len();

        // check for valid SOI
        if len < 2 || input[0] != 0xFF || input[1] != 0xD8 {
            return Ok(JpegModel {
                soi: Vec::new(),
                segments,
                eoi: Vec::new(),
            });
        }

        i = 2;

        while i + 3 < len {
            if input[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = input[i + 1];

            // occurs in DAT
            if marker == 0xFF {
                i += 1;
                continue;
            }

            // in EOI segment
            if marker == 0xD9 {
                break;
            }

            // marker without a lenght
            if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
                i += 2;
                continue;
            }

            let segment_length = u16::from_be_bytes([input[i + 2], input[i + 3]]) as usize;
            let end = i + 2 + segment_length;

            if end > len {
                break;
            }

            let segment_bytes = input[i..end].to_vec();

            match marker {
                0xE0..=0xEF => {
                    segments.push(JpegSegment::App(segment_bytes));
                }
                0xDB => {
                    segments.push(JpegSegment::Dqt(segment_bytes));
                }
                0xC0 | 0xC2 => {
                    segments.push(JpegSegment::Sof(segment_bytes));
                }
                0xC4 => {
                    segments.push(JpegSegment::Dht(segment_bytes));
                }
                0xDA => {
                    // Start of Scan
                    segments.push(JpegSegment::Sos(segment_bytes));

                    let mut image_data = Vec::new();
                    let mut j = end;

                    while j + 1 < len {
                        if input[j] == 0xFF && input[j + 1] == 0xD9 {
                            break;
                        }

                        // skip escaped 0xFF bytes (it must contain 0x00 after)
                        if input[j] == 0xFF && input[j + 1] == 0x00 {
                            image_data.push(0xFF);
                            j += 2;
                            continue;
                        }

                        image_data.push(input[j]);
                        j += 1;
                    }

                    segments.push(JpegSegment::Dat(image_data));
                    break;
                }
                _ => {
                    i += 1;
                    continue;
                }
            }

            i = end;
        }

        // while i < input.len() - 3 {
        //     // in the image data, xFF bytes are always followed by x00 so we want to
        //     // skip them, also skip the SOI/EOI
        //     if input[i] == 0xFF
        //         && !(input[i + 1] == 0x00
        //             || input[i + 1] == 0xFF
        //             || input[i + 1] == 0xD8
        //             || input[i + 1] == 0xD9)
        //     {
        //         let segment_length = u16::from_be_bytes([input[i + 2], input[i + 3]]) as usize;
        //         let segment = match (input[i], input[i + 1]) {
        //             (0xFF, 0xE0) => JpegSegment::App(input[i..i + segment_length + 2].to_vec()),
        //             (0xFF, 0xDB) => JpegSegment::Dqt(input[i..i + segment_length + 2].to_vec()),
        //             (0xFF, 0xC0 | 0xC2) => {
        //                 JpegSegment::Sof(input[i..i + segment_length + 2].to_vec())
        //             }
        //             (0xFF, 0xC4) => JpegSegment::Dht(input[i..i + segment_length + 2].to_vec()),
        //             (0xFF, 0xDA) => {
        //                 // get image data
        //                 let mut image_data = Vec::new();
        //                 let mut image_data_index = i + segment_length + 2;
        //                 while !(input[image_data_index] == 0xFF
        //                     && input[image_data_index + 1] == 0xD9)
        //                 {
        //                     image_data.push(input[image_data_index]);
        //                     image_data_index += 1;
        //                 }

        //                 let data_segment = JpegSegment::Dat(image_data);
        //                 let sos_segment =
        //                     JpegSegment::Sos(input[i..i + segment_length + 2].to_vec());
        //                 segments.push(sos_segment);
        //                 segments.push(data_segment);
        //                 break;
        //             }
        //             // TODO: more idiomatic way to ignore this
        //             _ => unimplemented!(),
        //         };
        //         segments.push(segment);
        //         i += segment_length + 2;
        //         continue;
        //     }
        //     i += 1;
        // }
        Ok(JpegModel {
            soi: vec![0xFF, 0xD8],
            segments,
            eoi: vec![0xFF, 0xD9],
        })
    }

    fn generate(model: Self::Model) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&model.soi);
        for seg in &model.segments {
            match seg {
                JpegSegment::App(data)
                | JpegSegment::Dqt(data)
                | JpegSegment::Sof(data)
                | JpegSegment::Dht(data)
                | JpegSegment::Sos(data)
                | JpegSegment::Dat(data) => {
                    bytes.extend_from_slice(data);
                }
            }
        }
        bytes.extend_from_slice(&model.eoi);

        Ok(bytes)
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    fn generate_corpus(rng: &mut SmallRng, corpus_dir: &Path) -> Result<()> {
        // create a default 16x16 jpg
        let width: u16 = 16;
        let height: u16 = 16;
        let mut default_img = image::ImageBuffer::new(u32::from(width), u32::from(height));

        for (x, y, pixel) in default_img.enumerate_pixels_mut() {
            let red = 3 * x as u8;
            let green = 3 * y as u8;
            let blue = 3 * (x + y) as u8;
            *pixel = image::Rgb([red, green, blue]);
        }

        default_img.save(corpus_dir.join("default.jpg"))?;
        let data = default_img.into_vec();

        // generate different qualities, densities, 3 byte/pixel color_types, and progressive encoding
        let color_types = [
            jpeg_encoder::ColorType::Luma,
            jpeg_encoder::ColorType::Rgb,
            jpeg_encoder::ColorType::Bgr,
            jpeg_encoder::ColorType::Ycbcr,
        ];

        for _ in 0..100 {
            let quality = rng.random_range(0..100);
            let progressive = rng.random_bool(0.5);
            let color_type_index = rng.random_range(0..color_types.len());
            let density = jpeg_encoder::Density::Inch {
                x: rng.random::<u16>(),
                y: rng.random::<u16>(),
            };

            let file_name = format!(
                "color-type={:?}_quality={quality}_progressive={progressive:?}_density={density:?}.jpg",
                color_types[color_type_index]
            );

            let mut encoder = jpeg_encoder::Encoder::new_file(corpus_dir.join(file_name), quality)?;
            encoder.set_progressive(progressive);
            encoder.set_density(density);
            encoder.encode(&data, width, height, color_types[color_type_index])?;
        }

        // vary the image dimensions with random pixels
        let dims = [
            (1, 1),
            (2, 2),
            (256, 256),
            (1024, 768),
            (1, 65535),
            (65535, 1),
        ];
        for (w, h) in dims {
            let mut img = image::ImageBuffer::new(w, h);
            for (_, _, pixel) in img.enumerate_pixels_mut() {
                *pixel = image::Rgb([rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>()]);
            }
            let file_name = format!("size-{w}x{h}.jpg");
            img.save(corpus_dir.join(file_name))?;
        }

        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn mutate(rng: &mut SmallRng, model: &mut Self::Model) -> Result<String> {
        let mut mutation_desc = String::new();
        if model.segments.is_empty() {
            return Ok(String::new());
        }

        match rng.random_range(0..=9) {
            0 => {
                let cut = rng.random_range(0..model.segments.len());
                model.segments.truncate(cut);
                mutation_desc = "truncated in half".to_string();
            }
            1 => {
                // remove EOI - last 2 bytes are a flag that represent the end
                // of the jpeg
                model.eoi.clear();
                mutation_desc = "removed EOI".to_string();
            }
            2 => {
                // corrupt SOI - replace the traditional jpeg start flag with a random byte
                let rand_byte = rng.random::<u8>();
                model.soi[1] = rand_byte;
                mutation_desc =
                    format!("corrupted SOI by inserting {rand_byte} into the second index");
            }
            3 => {
                // corrupt SOF - change the expected width/height of the file
                // xFF xC0 corresponds to baseline
                // xFF xC2 corresponds to progressive
                for seg in &mut model.segments {
                    if let JpegSegment::Sof(data) = seg {
                        if !data.is_empty() {
                            data[5] = rng.random::<u8>();
                            data[6] = rng.random::<u8>();
                            data[7] = rng.random::<u8>();
                            data[8] = rng.random::<u8>();
                        }
                    }
                }
                mutation_desc = "overwriting the expected width/height".to_string();
            }
            4 => {
                // byteflip non-header data, flip at most 2% of all non-header bytes
                let mut bytes = Self::generate(model.clone())?;
                let mutation_rate = rng.random_range(0.001..0.02);
                let total_byteflip_mutations = (bytes.len() as f64 * mutation_rate).ceil() as usize;

                // first collect all header indicies
                let mut header_indices = HashSet::new();
                for i in 0..bytes.len() - 1 {
                    if bytes[i] == 0xFF && bytes[i + 1] != 0x00 {
                        header_indices.insert(i);
                        header_indices.insert(i + 1);
                    }
                }

                for _ in 0..total_byteflip_mutations {
                    let mut index = rng.random_range(0..bytes.len() - 2);
                    while header_indices.contains(&index) {
                        index = rng.random_range(0..bytes.len() - 2);
                    }
                    bytes[index] ^= 1 << rng.random_range(0..8);
                }

                let new_model = Self::parse(&bytes)?;
                *model = new_model;
                mutation_desc = format!(
                    "byteflipping {:.2}% of nonheader data",
                    mutation_rate * 100.0,
                );
            }
            5 => {
                // add trailing garbage bytes at end
                let tail_length = rng.random_range(0..10_000);
                for _ in 0..tail_length {
                    model.eoi.push(rng.random::<u8>());
                }
                mutation_desc =
                    format!("adding {tail_length} random bytes at the end of the file data",);
            }
            6 => {
                // overwrite segment lengths, the two bytes after the segment header indicate the segment length
                let random_seg_idx = rng.random_range(..model.segments.len());
                let segment = &mut model.segments[random_seg_idx];

                let (data, segment_name) = match segment {
                    JpegSegment::App(v) => (v, "APP"),
                    JpegSegment::Dqt(v) => (v, "DQT"),
                    JpegSegment::Sof(v) => (v, "SOF"),
                    JpegSegment::Dht(v) => (v, "DHT"),
                    JpegSegment::Sos(v) => (v, "SOS"),
                    JpegSegment::Dat(v) => (v, "DAT"),
                };

                // generate two bytes from the same rng call
                let [high_byte, low_byte] = rng.random::<u16>().to_be_bytes();
                data[2] = high_byte;
                data[3] = low_byte;

                mutation_desc = format!(
                    "overwriting the segment length of the {segment_name} segment to be {:?}",
                    (u16::from(high_byte) >> 8) + u16::from(low_byte),
                );
            }
            7 => {
                // rearrange the segments
                let mut first = usize::MAX;
                let mut second = usize::MAX;
                while first == second {
                    first = rng.random_range(0..model.segments.len());
                    second = rng.random_range(0..model.segments.len());
                }
                let temp = model.segments[first].clone();
                model.segments[first] = model.segments[second].clone();
                model.segments[second] = temp;
                mutation_desc =
                    format!("swapping the two segments at positions {first} and {second}");
            }
            8 => {
                // alter dht tables
                for seg in &mut model.segments {
                    if let JpegSegment::Dht(data) = seg {
                        let byte_mutation_desc = mutate_bytes(rng, &mut data[5..]);
                        mutation_desc = format!("mutating one of the dhts: {byte_mutation_desc}");
                    }
                }
            }
            9 => {
                // alter dqt tables
                for seg in &mut model.segments {
                    if let JpegSegment::Dqt(data) = seg {
                        let byte_mutation_desc = mutate_bytes(rng, &mut data[5..]);
                        mutation_desc = format!("mutating one of the dqts: {byte_mutation_desc}");
                    }
                }
            }
            _ => unreachable!(),
        }
        Ok(mutation_desc)
    }
}
