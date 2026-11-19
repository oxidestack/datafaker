//! The part of `java.awt.image.BufferedImage` and `javax.imageio.ImageIO` that `Image` and `Fingerprint` use: images
//! of `TYPE_INT_RGB` and `TYPE_BYTE_GRAY`, `Graphics2D.fillRect` with an opaque colour, and `ImageIO.write(image,
//! format, stream)` through the JDK's own writers with their default parameters and metadata, byte for byte (pinned
//! by `tests/vectors/images.tsv`, recorded on JDK 25.0.2).

pub mod bmp;
pub mod gif;
pub mod jpeg;
pub mod png;
pub mod tiff;

/// A `BufferedImage` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    /// `TYPE_INT_RGB`: 8-bit red, green and blue samples packed in an `int`.
    IntRgb,
    /// `TYPE_BYTE_GRAY`: one 8-bit gray sample.
    ByteGray,
}

/// A `BufferedImage`: `bands` samples per pixel (3 for `TYPE_INT_RGB`, in R, G, B order; 1 for `TYPE_BYTE_GRAY`),
/// row by row from the top. A new image is all zero (black).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferedImage {
    width: usize,
    height: usize,
    kind: ImageType,
    samples: Vec<u8>,
}

impl BufferedImage {
    /// `new BufferedImage(width, height, type)`.
    pub fn new(width: usize, height: usize, kind: ImageType) -> BufferedImage {
        let bands = match kind {
            ImageType::IntRgb => 3,
            ImageType::ByteGray => 1,
        };
        BufferedImage { width, height, kind, samples: vec![0; width * height * bands] }
    }

    /// `BufferedImage.getWidth()`.
    pub fn width(&self) -> usize {
        self.width
    }

    /// `BufferedImage.getHeight()`.
    pub fn height(&self) -> usize {
        self.height
    }

    /// `BufferedImage.getType()`.
    pub fn kind(&self) -> ImageType {
        self.kind
    }

    /// Samples per pixel.
    pub fn bands(&self) -> usize {
        match self.kind {
            ImageType::IntRgb => 3,
            ImageType::ByteGray => 1,
        }
    }

    /// Row `y`'s samples.
    pub fn row(&self, y: usize) -> &[u8] {
        let n = self.width * self.bands();
        &self.samples[y * n..(y + 1) * n]
    }

    /// The samples of pixel `(x, y)`.
    pub fn pixel(&self, x: usize, y: usize) -> &[u8] {
        let b = self.bands();
        let i = (y * self.width + x) * b;
        &self.samples[i..i + b]
    }

    /// `getRaster().setSample(x, y, 0, value)` of a `TYPE_BYTE_GRAY` image.
    pub fn set_gray(&mut self, x: usize, y: usize, value: u8) {
        debug_assert_eq!(self.kind, ImageType::ByteGray);
        self.samples[y * self.width + x] = value;
    }

    /// `Graphics2D.setColor(new Color(r, g, b))` then `fillRect(x, y, w, h)` on a `TYPE_INT_RGB` image, clipped to
    /// the image.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, rgb: [u8; 3]) {
        debug_assert_eq!(self.kind, ImageType::IntRgb);
        let (x1, y1) = ((x + w).min(self.width), (y + h).min(self.height));
        for yy in y.min(self.height)..y1 {
            for xx in x.min(self.width)..x1 {
                let i = (yy * self.width + xx) * 3;
                self.samples[i..i + 3].copy_from_slice(&rgb);
            }
        }
    }
}

/// `ImageIO.write(image, formatName, stream)`'s bytes for the formats Datafaker writes (`BMP`, `GIF`, `JPEG`, `PNG`, `TIFF`; the format
/// name is matched as `ImageIO` does, ignoring case). `None` for a format whose writer is not ported.
pub fn write(image: &BufferedImage, format_name: &str) -> Option<Vec<u8>> {
    match format_name.to_ascii_lowercase().as_str() {
        "png" => Some(png::write(image)),
        "bmp" if image.kind() == ImageType::IntRgb => Some(bmp::write(image)),
        "gif" if image.kind() == ImageType::IntRgb => Some(gif::write(image)),
        "jpeg" | "jpg" if image.kind() == ImageType::IntRgb => Some(jpeg::write(image)),
        "tiff" if image.kind() == ImageType::IntRgb => Some(tiff::write(image)),
        _ => None,
    }
}

/// Big-endian `int` (`ImageOutputStream.writeInt` in its default byte order).
fn be32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}
