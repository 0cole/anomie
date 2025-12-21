use anyhow::Result;
use image;
use jpeg_encoder;
use log::info;
use rand::{Rng, rngs::SmallRng, seq::IteratorRandom};
use std::{fs, path::PathBuf};

use crate::{
    analysis::analyze_result,
    errors::ExitStatus,
    mutate::mutate_jpeg,
    target::run_target_file,
    types::{Config, StructuredInput},
};

#[derive(Debug, Clone)]
pub struct JpegObject {
    pub soi: Vec<u8>,
    pub segments: Vec<JpegSegment>,
    pub eoi: Vec<u8>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub enum JpegSegment {
    APP(Vec<u8>),
    DQT(Vec<u8>),
    SOF(Vec<u8>),
    DHT(Vec<u8>),
    SOS(Vec<u8>),
    DAT(Vec<u8>), // image data
    Other(u8, Vec<u8>),
}

impl JpegObject {
    pub fn new(file: &PathBuf) -> Result<JpegObject> {
        let bytes: Vec<u8> = fs::read(file)?;
        let mut segments = Vec::new();

        let mut i = 0;
        while i < bytes.len() - 3 {
            // in the image data, xFF bytes are always followed by x00 so we want to
            // skip them, also skip the SOI/EOI
            if bytes[i] == 0xFF
                && !(bytes[i + 1] == 0x00
                    || bytes[i + 1] == 0xFF
                    || bytes[i + 1] == 0xD8
                    || bytes[i + 1] == 0xD9)
            {
                let segment_length = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
                let segment = match (bytes[i], bytes[i + 1]) {
                    (0xFF, 0xE0) => JpegSegment::APP(bytes[i..i + segment_length + 2].to_vec()),
                    (0xFF, 0xDB) => JpegSegment::DQT(bytes[i..i + segment_length + 2].to_vec()),
                    (0xFF, 0xC0 | 0xC2) => {
                        JpegSegment::SOF(bytes[i..i + segment_length + 2].to_vec())
                    }
                    (0xFF, 0xC4) => JpegSegment::DHT(bytes[i..i + segment_length + 2].to_vec()),
                    (0xFF, 0xDA) => {
                        // get image data
                        let mut image_data = Vec::new();
                        let mut image_data_index = i + segment_length + 2;
                        while !(bytes[image_data_index] == 0xFF
                            && bytes[image_data_index + 1] == 0xD9)
                        {
                            image_data.push(bytes[image_data_index]);
                            image_data_index += 1;
                        }

                        let data_segment = JpegSegment::DAT(image_data);
                        let sos_segment =
                            JpegSegment::SOS(bytes[i..i + segment_length + 2].to_vec());
                        segments.push(sos_segment);
                        segments.push(data_segment);
                        break;
                    }
                    // TODO: more idiomatic way to ignore this
                    _ => unimplemented!(),
                };
                segments.push(segment);
                i += segment_length + 2;
                continue;
            }
            i += 1;
        }
        Ok(JpegObject {
            soi: vec![0xFF, 0xD8],
            segments,
            eoi: vec![0xFF, 0xD9],
        })
    }

    pub fn write_to_file(&self, file: &'static str) -> Result<()> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&self.soi);
        for seg in &self.segments {
            match seg {
                JpegSegment::APP(data)
                | JpegSegment::DQT(data)
                | JpegSegment::SOF(data)
                | JpegSegment::DHT(data)
                | JpegSegment::SOS(data)
                | JpegSegment::DAT(data)
                | JpegSegment::Other(_, data) => {
                    bytes.extend_from_slice(data);
                }
            }
        }
        bytes.extend_from_slice(&self.eoi);

        fs::write(file, bytes)?;
        Ok(())
    }

    pub fn random_dht_mut(&mut self, rng: &mut SmallRng) -> Option<&mut Vec<u8>> {
        self.segments
            .iter_mut()
            .filter_map(|seg| {
                if let JpegSegment::DQT(data) = seg {
                    Some(data)
                } else {
                    None
                }
            })
            .choose(rng)
    }

    pub fn random_dqt_mut(&mut self, rng: &mut SmallRng) -> Option<&mut Vec<u8>> {
        self.segments
            .iter_mut()
            .filter_map(|seg| {
                if let JpegSegment::DQT(data) = seg {
                    Some(data)
                } else {
                    None
                }
            })
            .choose(rng)
    }

    pub fn soi_mut(&mut self) -> Option<&mut Vec<u8>> {
        if !self.soi.is_empty() {
            return Some(&mut self.soi);
        }
        None
    }

    pub fn eoi_mut(&mut self) -> Option<&mut Vec<u8>> {
        if !self.eoi.is_empty() {
            return Some(&mut self.eoi);
        }
        None
    }

    pub fn app_mut(&mut self) -> Option<&mut Vec<u8>> {
        for seg in &mut self.segments {
            if let JpegSegment::APP(data) = seg {
                return Some(data);
            }
        }
        None
    }

    pub fn sof_mut(&mut self) -> Option<&mut Vec<u8>> {
        for seg in &mut self.segments {
            if let JpegSegment::SOF(data) = seg {
                return Some(data);
            }
        }
        None
    }

    pub fn sos_mut(&mut self) -> Option<&mut Vec<u8>> {
        for seg in &mut self.segments {
            if let JpegSegment::SOS(data) = seg {
                return Some(data);
            }
        }
        None
    }

    pub fn dat_mut(&mut self) -> Option<&mut Vec<u8>> {
        for seg in &mut self.segments {
            if let JpegSegment::DAT(data) = seg {
                return Some(data);
            }
        }
        None
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn generate_corpus(rng: &mut SmallRng, corpus_dir: &str) -> Result<()> {
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

    default_img.save(corpus_dir.to_string() + "default.jpg")?;
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
            "{corpus_dir}color-type={:?}_quality={quality}_progressive={progressive:?}_density={density:?}.jpg",
            color_types[color_type_index]
        );
        let mut encoder = jpeg_encoder::Encoder::new_file(file_name, quality)?;
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
        let file_name = format!("{corpus_dir}size-{w}x{h}.jpg");
        img.save(file_name)?;
    }

    Ok(())
}

pub fn fuzz_jpeg(config: &mut Config) -> Result<()> {
    info!("Beginning jpeg fuzzing");

    let corpus_jpeg_dir = "corpus/jpg/";
    generate_corpus(&mut config.rng, corpus_jpeg_dir)?;

    // create a vec of every .jpg in the corpus dir
    let jpgs: Vec<PathBuf> = fs::read_dir(corpus_jpeg_dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()?.to_str()?.eq_ignore_ascii_case("jpg") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let mutated_file_path = "temp/mutated.jpg";
    let args: Vec<String> = if config.bin_args == [""] {
        vec![mutated_file_path.to_string()]
    } else {
        let mut v = config.bin_args.clone();
        v.push(mutated_file_path.to_string());
        v
    };

    for id in 0..config.iterations {
        let file_num = config.rng.random_range(0..jpgs.len());
        let file = &jpgs[file_num];
        mutate_jpeg(&mut config.rng, file)?;

        let result = run_target_file(config, &file.to_string_lossy().into_owned())
            .unwrap_or(ExitStatus::ExitCode(0));

        let structured_input = StructuredInput::FileInput {
            path: PathBuf::from(&mutated_file_path),
            extension: "jpg".to_string(),
        };

        analyze_result(
            &config.report_path,
            &mut config.crash_stats,
            result,
            id,
            structured_input,
        )?;
    }

    Ok(())
}
