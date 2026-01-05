use std::{
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use crc32fast::Hasher;
use log::{debug, warn};
use rand::Rng;

use crate::mutations::png as png_mutations;

use super::template::FileFormat;

pub struct Png;

pub struct PngModel {
    pub signature: [u8; 8],
    pub chunks: Vec<Chunk>,
}

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[derive(Debug, PartialEq, Eq)]
pub enum Chunk {
    Ihdr(Ihdr, ChunkCrc),
    Idat(Vec<u8>, ChunkCrc),
    Iend(ChunkCrc),
    Ancillary(RawChunk),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ihdr {
    pub width: u32,
    pub height: u32,
    pub depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChunkCrc {
    pub crc: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawChunk {
    pub length: u32,
    pub chunk_type: [u8; 4],
    pub data: Vec<u8>,
    pub crc: u32,
}

// reads a chunk from a bytestream. parses it and returns a raw chunk.
// the raw chunk still needs to be interreted
fn parse_chunk(cursor: &mut Cursor<&[u8]>) -> Result<RawChunk> {
    let length = cursor.read_u32::<BigEndian>()?;
    let mut chunk_type = [0u8; 4];
    cursor.read_exact(&mut chunk_type)?;

    let mut data = vec![0; length as usize];
    cursor.read_exact(&mut data)?;

    let crc = cursor.read_u32::<BigEndian>()?;

    Ok(RawChunk {
        length,
        chunk_type,
        data,
        crc,
    })
}

fn bytes_to_ihdr(data: &[u8]) -> Result<Ihdr> {
    Ok(Ihdr {
        width: u32::from_be_bytes(data[0..4].try_into()?),
        height: u32::from_be_bytes(data[4..8].try_into()?),
        depth: data[8],
        color_type: data[9],
        compression_method: data[10],
        filter_method: data[11],
        interlace_method: data[12],
    })
}

fn ihdr_to_bytes(ihdr: &Ihdr) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(&ihdr.width.to_be_bytes());
    bytes.extend_from_slice(&ihdr.height.to_be_bytes());
    bytes.extend_from_slice(&ihdr.depth.to_be_bytes());
    bytes.extend_from_slice(&ihdr.color_type.to_be_bytes());
    bytes.extend_from_slice(&ihdr.compression_method.to_be_bytes());
    bytes.extend_from_slice(&ihdr.filter_method.to_be_bytes());
    bytes.extend_from_slice(&ihdr.interlace_method.to_be_bytes());
    bytes
}

fn minimal_idat(scanline: &[u8]) -> Result<Vec<u8>> {
    let len = u16::try_from(scanline.len())?;
    let nlen = !len;

    let mut out = Vec::new();
    out.extend_from_slice(&[0x78, 0x01]); // zlib header
    out.push(0x01); // final, uncompressed block
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&nlen.to_le_bytes());
    out.extend_from_slice(scanline);

    // adler32
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in scanline {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    let adler = (b << 16) | a;
    out.extend_from_slice(&adler.to_be_bytes());

    Ok(out)
}

pub fn interpret_chunk(raw: RawChunk) -> Result<Chunk> {
    match &raw.chunk_type {
        b"IHDR" => Ok(Chunk::Ihdr(
            bytes_to_ihdr(&raw.data)?,
            ChunkCrc { crc: raw.crc },
        )),
        b"IDAT" => Ok(Chunk::Idat(raw.data, ChunkCrc { crc: raw.crc })),
        b"IEND" => Ok(Chunk::Iend(ChunkCrc { crc: raw.crc })),
        _ => Ok(Chunk::Ancillary(raw)),
    }
}

// calculates the length and crc. extends byte_stream
fn write_chunk(byte_stream: &mut Vec<u8>, ty: &[u8], data: &[u8], opt_crc: Option<u32>) {
    let len = u32::try_from(data.len()).unwrap();
    byte_stream.extend_from_slice(&len.to_be_bytes());
    byte_stream.extend_from_slice(ty);
    byte_stream.extend_from_slice(data);

    if let Some(crc) = opt_crc {
        byte_stream.extend_from_slice(&crc.to_be_bytes());
    } else {
        let crc = png_crc(ty, data);
        byte_stream.extend_from_slice(&crc.to_be_bytes());
    }
}

fn png_crc(ty: &[u8], data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(ty);
    hasher.update(data);
    hasher.finalize()
}

fn write_png(path: PathBuf, mut model: PngModel, normalize: bool) -> Result<()> {
    if normalize {
        normalize_png(&mut model);
    }
    let bytes = Png::generate(model)?;
    std::fs::write(path, &bytes)?;
    Ok(())
}

fn normalize_png(model: &mut PngModel) {
    for chunk in &mut model.chunks {
        let (ty, data, crc) = match chunk {
            Chunk::Ihdr(data, crc) => (b"IHDR", &mut ihdr_to_bytes(data), crc),
            Chunk::Idat(data, crc) => (b"IDAT", data, crc),
            Chunk::Iend(crc) => (b"IEND", &mut [].to_vec(), crc),
            Chunk::Ancillary(raw) => (
                &raw.chunk_type,
                &mut raw.data,
                &mut ChunkCrc { crc: raw.crc },
            ),
        };

        crc.crc = png_crc(ty, data);
    }
}

impl FileFormat for Png {
    type Model = PngModel;
    const EXT: &'static str = "png";

    fn parse(input: &[u8]) -> Result<Self::Model> {
        let mut cursor = Cursor::new(input);

        let mut signature = [0u8; 8];
        cursor.read_exact(&mut signature)?;

        if signature != PNG_SIGNATURE {
            warn!("PNG signature is invalid");
        }

        let mut chunks: Vec<Chunk> = Vec::new();
        while cursor.position() < input.len().try_into().unwrap() {
            let raw_chunk = parse_chunk(&mut cursor)?;
            let chunk: Chunk = interpret_chunk(raw_chunk)?;
            chunks.push(chunk);
        }

        Ok(PngModel { signature, chunks })
    }

    fn generate(model: Self::Model) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&model.signature);

        for chunk in &model.chunks {
            match chunk {
                Chunk::Ihdr(data, crc) => {
                    write_chunk(&mut bytes, b"IHDR", &ihdr_to_bytes(data), Some(crc.crc));
                }
                Chunk::Idat(data, crc) => write_chunk(&mut bytes, b"IDAT", data, Some(crc.crc)),
                Chunk::Iend(crc) => write_chunk(&mut bytes, b"IEND", &[], Some(crc.crc)),
                Chunk::Ancillary(raw) => {
                    write_chunk(&mut bytes, &raw.chunk_type, &raw.data, Some(raw.crc));
                }
            }
        }

        Ok(bytes)
    }

    fn generate_corpus(
        _rng: &mut rand::prelude::SmallRng,
        corpus_dir: &std::path::Path,
    ) -> Result<()> {
        // name, color type, bit depth, bpp
        let variations = [
            ("indexed", 3, 8, 1),
            ("gray", 0, 8, 1),
            ("gray16", 0, 16, 2),
            ("rgb", 2, 8, 3),
            ("rgba", 6, 8, 4),
            ("interlace", 2, 8, 3),
        ];

        for (name, color_type, depth, bpp) in variations {
            let mut model = PngModel {
                signature: PNG_SIGNATURE,
                chunks: Vec::new(),
            };

            // ==== generate IHDR ====
            let ihdr = Ihdr {
                width: 1,
                height: 1,
                depth,
                color_type,
                compression_method: 0,
                filter_method: 0,
                interlace_method: u8::from(name == "interlace"),
            };
            let ihdr_bytes = ihdr_to_bytes(&ihdr);
            let ihdr_crc = png_crc(b"IHDR", &ihdr_bytes);
            let ihdr_raw_chunk = RawChunk {
                length: 13,
                chunk_type: *b"IHDR",
                data: ihdr_bytes,
                crc: ihdr_crc,
            };
            model.chunks.push(interpret_chunk(ihdr_raw_chunk)?);

            // ==== generate IDAT ====
            // each scanline corresponds to a row. since this is 1x1, we only have one pixel
            // filter bytes:
            //   0: nothing
            //   1: sub
            //   2: up
            //   3: average
            //   4: paeth
            let mut scanline = Vec::new();
            scanline.push(0u8); // filter byte
            // since these are all 1x1, we can extend it 'bpp' times
            for _ in 0..bpp {
                scanline.extend_from_slice(&[0u8]);
            }

            let idat_bytes = minimal_idat(&scanline)?;
            let idat_crc = png_crc(b"IDAT", &idat_bytes);
            let idat_raw_chunk = RawChunk {
                length: u32::try_from(idat_bytes.len())?,
                chunk_type: *b"IDAT",
                data: idat_bytes,
                crc: idat_crc,
            };
            model.chunks.push(interpret_chunk(idat_raw_chunk)?);

            // ==== add IEND ====
            model.chunks.push(Chunk::Iend(ChunkCrc { crc: 0 }));

            let path = corpus_dir.join(format!("{name}.png"));
            write_png(path, model, true)?;
            debug!("created {name}.png");
        }

        debug!("finished generating corpus");
        Ok(())
    }

    fn mutate(rng: &mut rand::prelude::SmallRng, model: &mut Self::Model) -> Result<String> {
        if model.chunks.is_empty() {
            return Ok(String::new());
        }

        let mutation_desc = match rng.random_range(0..9) {
            0 => png_mutations::remove_ihdr(model),
            1 => png_mutations::remove_idat(model),
            2 => png_mutations::remove_iend(model),
            3 => png_mutations::change_png_dims(model, rng),
            4 => png_mutations::change_depth(model, rng),
            5 => png_mutations::change_color_type(model, rng),
            6 => png_mutations::change_compression_method(model, rng),
            7 => png_mutations::change_filter_method(model, rng),
            8 => png_mutations::change_interlace_method(model, rng),
            9 => png_mutations::xor_crc(model, rng),
            10 => png_mutations::zero_crc(model, rng),
            _ => {
                unreachable!()
            }
        };

        Ok(mutation_desc)
    }
}
