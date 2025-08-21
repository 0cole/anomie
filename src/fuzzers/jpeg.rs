use image;
use jpeg_encoder;
use log::info;
use rand::{
    Rng,
    rngs::{SmallRng, ThreadRng},
};
use std::{fs, path::PathBuf};

use crate::{
    analysis::analyze_result,
    errors::ExitStatus,
    mutate::mutate_jpeg,
    target::run_target_file,
    types::{Config, StructuredInput},
    utils::clean_up,
};

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn generate_corpus(rng: &mut SmallRng, corpus_dir: &str) {
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

    default_img
        .save(corpus_dir.to_string() + "default.jpg")
        .unwrap();
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
        let mut encoder = jpeg_encoder::Encoder::new_file(file_name, quality).unwrap();
        encoder.set_progressive(progressive);
        encoder.set_density(density);
        encoder
            .encode(&data, width, height, color_types[color_type_index])
            .unwrap();
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
        img.save(file_name).unwrap();
    }
}

pub fn fuzz_jpeg(config: &mut Config) {
    info!("Beginning jpeg fuzzing");

    let corpus_jpeg_dir = "corpus/jpeg/";
    generate_corpus(&mut config.rng, corpus_jpeg_dir);

    // create a vec of every .jpg in the corpus dir
    let jpgs: Vec<PathBuf> = fs::read_dir(corpus_jpeg_dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()?.to_str()?.eq_ignore_ascii_case("jpg") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let mutated_file = "mutated.jpg";
    let args: Vec<String> = if config.bin_args == [""] {
        vec![mutated_file.to_string()]
    } else {
        let mut v = config.bin_args.clone();
        v.push(mutated_file.to_string());
        v
    };

    for id in 0..config.max_iterations {
        let file_num = config.rng.random_range(0..jpgs.len());
        let file = &jpgs[file_num];
        mutate_jpeg(&mut config.rng, file).unwrap();

        let result = run_target_file(&args, &config.bin_path).unwrap_or(ExitStatus::ExitCode(0));
        let structured_input =
            StructuredInput::FileInput(mutated_file.to_string(), "jpg".to_string());
        analyze_result(&config.report_path, result, id, structured_input);
    }

    // clean_up(corpus_jpeg_dir, "jpg");
}
