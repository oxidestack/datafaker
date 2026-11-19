//! `com.sun.imageio.plugins.tiff.TIFFImageWriter` with no write parameters, for `TYPE_INT_RGB` images: a big-endian
//! file with one IFD right after the header, of the twelve fields the writer sets for such an image (in tag order,
//! a value over four bytes written after the IFD on a four-byte boundary, a width, height or row count below 65536
//! as a `SHORT`), uncompressed RGB strips of `max(8192 / bytesPerRow, 8)` rows (`DEFAULT_BYTES_PER_STRIP`) after
//! the values, the last one holding the rows left.

use super::{BufferedImage, be32};

const DEFAULT_BYTES_PER_STRIP: usize = 8192;
const SHORT: u16 = 3;
const LONG: u16 = 4;
const RATIONAL: u16 = 5;

/// A field: tag, type and its values (as big-endian bytes).
struct Field {
    tag: u16,
    kind: u16,
    count: u32,
    value: Vec<u8>,
}

fn shorts(tag: u16, values: &[u16]) -> Field {
    Field { tag, kind: SHORT, count: values.len() as u32, value: values.iter().flat_map(|v| v.to_be_bytes()).collect() }
}

fn longs(tag: u16, values: &[u32]) -> Field {
    Field { tag, kind: LONG, count: values.len() as u32, value: values.iter().flat_map(|v| v.to_be_bytes()).collect() }
}

/// `new TIFFField(tag, value)`: a `SHORT` below 65536, else a `LONG`.
fn int(tag: u16, value: usize) -> Field {
    if value < 65536 { shorts(tag, &[value as u16]) } else { longs(tag, &[value as u32]) }
}

fn rational(tag: u16, numerator: u32, denominator: u32) -> Field {
    let mut value = numerator.to_be_bytes().to_vec();
    value.extend_from_slice(&denominator.to_be_bytes());
    Field { tag, kind: RATIONAL, count: 1, value }
}

/// `ImageIO.write(image, "tiff", stream)`.
pub fn write(image: &BufferedImage) -> Vec<u8> {
    let (w, h) = (image.width(), image.height());
    let bytes_per_row = w * 3;
    let rows_per_strip = (DEFAULT_BYTES_PER_STRIP / bytes_per_row).max(1).max(8).min(h);
    let strips = h.div_ceil(rows_per_strip);
    let counts: Vec<u32> = (0..strips).map(|i| (rows_per_strip.min(h - i * rows_per_strip) * bytes_per_row) as u32).collect();
    let mut fields = vec![
        int(256, w),
        int(257, h),
        shorts(258, &[8, 8, 8]),
        shorts(259, &[1]),
        shorts(262, &[2]),
        longs(273, &vec![0; strips]),
        shorts(277, &[3]),
        int(278, rows_per_strip),
        longs(279, &counts),
        rational(282, 1, 1),
        rational(283, 1, 1),
        shorts(296, &[1]),
    ];
    // Lay the values out: after the IFD (count, entries, next-IFD pointer), each on a four-byte boundary.
    let mut next_space = 8 + 2 + 12 * fields.len() + 4;
    let mut offsets = vec![None; fields.len()];
    for (i, f) in fields.iter().enumerate() {
        if f.value.len() > 4 {
            next_space = (next_space + 3) & !3;
            offsets[i] = Some(next_space);
            next_space += f.value.len();
        }
    }
    // The strips follow the values.
    let mut pos = next_space;
    let strip_offsets: Vec<u32> = counts
        .iter()
        .map(|c| {
            let at = pos as u32;
            pos += *c as usize;
            at
        })
        .collect();
    fields[5] = longs(273, &strip_offsets);
    let mut out = Vec::with_capacity(pos);
    out.extend_from_slice(b"MM\0\x2a");
    be32(&mut out, 8);
    out.extend_from_slice(&(fields.len() as u16).to_be_bytes());
    for (f, offset) in fields.iter().zip(&offsets) {
        out.extend_from_slice(&f.tag.to_be_bytes());
        out.extend_from_slice(&f.kind.to_be_bytes());
        be32(&mut out, f.count);
        match offset {
            Some(o) => be32(&mut out, *o as u32),
            None => {
                let mut v = f.value.clone();
                v.resize(4, 0);
                out.extend_from_slice(&v);
            }
        }
    }
    be32(&mut out, 0);
    for (f, offset) in fields.iter().zip(&offsets) {
        if let Some(o) = offset {
            out.resize(*o, 0);
            out.extend_from_slice(&f.value);
        }
    }
    for y in 0..h {
        out.extend_from_slice(image.row(y));
    }
    out
}
