//! `com.sun.imageio.plugins.bmp.BMPImageWriter` with no write parameters, for `TYPE_INT_RGB` images as Datafaker
//! writes them: a `BITMAPINFOHEADER` (version 3) file, 24 bits per pixel, `BI_RGB`, no
//! resolution, rows bottom-up in B, G, R order, each padded to four bytes.

use super::BufferedImage;

fn le32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// `ImageIO.write(image, "bmp", stream)`.
pub fn write(image: &BufferedImage) -> Vec<u8> {
    let (w, h) = (image.width(), image.height());
    let row = (w * 3).div_ceil(4) * 4;
    let image_size = row * h;
    let mut out = Vec::with_capacity(54 + image_size);
    out.extend_from_slice(b"BM");
    le32(&mut out, (54 + image_size) as u32);
    le32(&mut out, 0);
    le32(&mut out, 54);
    le32(&mut out, 40);
    le32(&mut out, w as u32);
    le32(&mut out, h as u32);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    le32(&mut out, 0);
    le32(&mut out, image_size as u32);
    for _ in 0..4 {
        le32(&mut out, 0);
    }
    for y in (0..h).rev() {
        let start = out.len();
        for x in 0..w {
            let p = image.pixel(x, y);
            out.extend_from_slice(&[p[2], p[1], p[0]]);
        }
        out.resize(start + row, 0);
    }
    out
}
