//! `com.sun.imageio.plugins.png.PNGImageWriter` with no write parameters and default metadata: the signature,
//! `IHDR` (8-bit samples, gray or RGB by the band count, no interlacing), the image data deflated by
//! `Deflater(4)` (`DEFAULT_COMPRESSION_LEVEL`) and split into `IDAT` chunks of 32768 bytes, and `IEND`.
//! `RowFilter.filterRow` filters only palette images, so every row of these is filter type 0 (none).

use super::{BufferedImage, be32};
use crate::java::deflater::{Crc32, deflate};

const DEFAULT_COMPRESSION_LEVEL: usize = 4;
const IDAT_CHUNK_LENGTH: usize = 32768;

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    be32(out, data.len() as u32);
    let mut crc = Crc32::new();
    crc.update(kind);
    crc.update(data);
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    be32(out, crc.value());
}

/// `ImageIO.write(image, "png", stream)`.
pub fn write(image: &BufferedImage) -> Vec<u8> {
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let color_type = if image.bands() == 1 { 0 } else { 2 };
    let mut ihdr = Vec::with_capacity(13);
    be32(&mut ihdr, image.width() as u32);
    be32(&mut ihdr, image.height() as u32);
    ihdr.extend_from_slice(&[8, color_type, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    let mut raw = Vec::with_capacity((image.width() * image.bands() + 1) * image.height());
    for y in 0..image.height() {
        raw.push(0);
        raw.extend_from_slice(image.row(y));
    }
    let data = deflate(DEFAULT_COMPRESSION_LEVEL, &raw);
    for piece in data.chunks(IDAT_CHUNK_LENGTH) {
        chunk(&mut out, b"IDAT", piece);
    }
    chunk(&mut out, b"IEND", &[]);
    out
}
