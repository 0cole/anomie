use rand::{Rng, seq::IndexedRandom};

use crate::formats::png::{Chunk, PngModel};

pub fn remove_ihdr(model: &mut PngModel) -> String {
    model.chunks.retain(|c| !matches!(c, Chunk::Ihdr(_)));
    "removed ihdr".to_string()
}

pub fn remove_idat(model: &mut PngModel) -> String {
    model.chunks.retain(|c| !matches!(c, Chunk::Idat(_)));
    "removed idat".to_string()
}

pub fn remove_iend(model: &mut PngModel) -> String {
    model.chunks.retain(|c| c != &Chunk::Iend);
    "removed iend".to_string()
}

pub fn change_png_dims(model: &mut PngModel, rng: &mut rand::prelude::SmallRng) -> String {
    let possibilities = &[
        0,
        1,
        2,
        7,
        8,
        9,
        15,
        16,
        17,
        31,
        32,
        33,
        63,
        64,
        65,
        (2 << 8) - 1,
        2 << 8,
        (2 << 8) + 1,
        (2 << 16) - 1,
        2 << 16,
        (2 << 16) + 1,
        i32::MAX as u32 - 1,
        i32::MAX as u32,
        i32::MAX as u32 + 1,
        u32::MAX - 1,
        u32::MAX,
    ];

    let rand_width = possibilities.choose(rng).unwrap_or(&0u32);
    let rand_height = possibilities.choose(rng).unwrap_or(&0u32);

    let mut changed = false;
    for chunk in &mut model.chunks {
        if let Chunk::Ihdr(ihdr) = chunk {
            ihdr.width = *rand_width;
            ihdr.height = *rand_height;
            changed = true;
        }
    }

    if changed {
        format!("changed width/height to {rand_width}/{rand_height}").to_string()
    } else {
        "could not find ihdr".to_string()
    }
}

pub fn change_depth(model: &mut PngModel, rng: &mut rand::prelude::SmallRng) -> String {
    // 0 and 128 are not supported under any circumstances, the rest are possible
    let possibilities = &[0, 1, 2, 4, 8, 16, 32, 64, 128];
    let rand_depth = possibilities.choose(rng).unwrap_or(&1u8);

    let mut changed = false;
    for chunk in &mut model.chunks {
        if let Chunk::Ihdr(ihdr) = chunk {
            ihdr.depth = *rand_depth;
            changed = true;
        }
    }

    if changed {
        format!("changed depth to {rand_depth}").to_string()
    } else {
        "could not find ihdr".to_string()
    }
}

pub fn change_color_type(model: &mut PngModel, rng: &mut rand::prelude::SmallRng) -> String {
    // the only valid color types are 0, 2, 3, 4, and 6
    let possibilities = &[0, 1, 2, 3, 4, 6, 8, 32, u8::MAX - 1, u8::MAX];
    let rand_ctype = possibilities.choose(rng).unwrap_or(&1u8);

    let mut changed = false;
    for chunk in &mut model.chunks {
        if let Chunk::Ihdr(ihdr) = chunk {
            ihdr.color_type = *rand_ctype;
            changed = true;
        }
    }

    if changed {
        format!("changed color type to {rand_ctype}").to_string()
    } else {
        "could not find ihdr".to_string()
    }
}

pub fn change_compression_method(
    model: &mut PngModel,
    rng: &mut rand::prelude::SmallRng,
) -> String {
    // the only valid integer is zero, so replace it with any other u8
    let rand_u8 = rng.random::<u8>();
    let mut changed = false;
    for chunk in &mut model.chunks {
        if let Chunk::Ihdr(ihdr) = chunk {
            ihdr.color_type = rng.random::<u8>();
            changed = true;
        }
    }

    if changed {
        format!("changed comperssion method to a nonzero integer {rand_u8}").to_string()
    } else {
        "could not find ihdr".to_string()
    }
}

pub fn change_filter_method(model: &mut PngModel, rng: &mut rand::prelude::SmallRng) -> String {
    // the only valid integer is zero, so replace it with any other u8
    let rand_u8 = rng.random::<u8>();
    let mut changed = false;
    for chunk in &mut model.chunks {
        if let Chunk::Ihdr(ihdr) = chunk {
            ihdr.filter_method = rng.random::<u8>();
            changed = true;
        }
    }

    if changed {
        format!("changed filter method to a nonzero integer {rand_u8}").to_string()
    } else {
        "could not find ihdr".to_string()
    }
}

pub fn change_interlace_method(model: &mut PngModel, rng: &mut rand::prelude::SmallRng) -> String {
    // the only valid interlace methods are 0 and 1
    let possibilities = &[0, 1, 2, 3, u8::MAX];
    let rand_interlace_method = possibilities.choose(rng).unwrap_or(&0u8);

    let mut changed = false;
    for chunk in &mut model.chunks {
        if let Chunk::Ihdr(ihdr) = chunk {
            ihdr.interlace_method = *rand_interlace_method;
            changed = true;
        }
    }

    if changed {
        format!("changed color type to {rand_interlace_method}").to_string()
    } else {
        "could not find ihdr".to_string()
    }
}
