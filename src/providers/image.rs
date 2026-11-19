//! `net.datafaker.providers.base.Image`, with `ImageType`, `Base64ImageRuleConfig` and `ImageBuilder`.

use super::s;
use crate::dispatch::{Method, P};
use crate::java::imageio::{self, BufferedImage};
use crate::{Error, Faker, Result};

const DEFAULT_WIDTH: i32 = 256;
const DEFAULT_HEIGHT: i32 = 256;

/// `Image.ImageType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageType {
    /// `BMP`.
    Bmp,
    /// `GIF`.
    Gif,
    /// `JPEG`.
    Jpeg,
    /// `PNG`.
    Png,
    /// `SVG`.
    Svg,
    /// `TIFF`.
    Tiff,
}

impl ImageType {
    /// `values()`.
    pub const VALUES: [ImageType; 6] = [ImageType::Bmp, ImageType::Gif, ImageType::Jpeg, ImageType::Png, ImageType::Svg, ImageType::Tiff];

    /// `name()`.
    pub fn name(self) -> &'static str {
        ["BMP", "GIF", "JPEG", "PNG", "SVG", "TIFF"][self as usize]
    }

    /// `getMimeType()`.
    pub fn mime_type(self) -> &'static str {
        ["image/bmp", "image/gif", "image/jpeg", "image/png", "image/svg+xml", "image/tiff"][self as usize]
    }
}

/// `Image.Base64ImageRuleConfig`: a record, so any width and height (a raster image fails on one below 1, as
/// `BufferedImage` does; an SVG has no boxes then).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base64ImageRuleConfig {
    /// The image format.
    pub image_type: ImageType,
    /// The width in pixels.
    pub width: i32,
    /// The height in pixels.
    pub height: i32,
}

impl Base64ImageRuleConfig {
    /// A configuration for a `width` by `height` image of `image_type`.
    pub fn new(image_type: ImageType, width: i32, height: i32) -> Self {
        Base64ImageRuleConfig { image_type, width, height }
    }
}

/// `Image.ImageBuilder`: PNG, 256 × 256 unless set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageBuilder {
    image_type: ImageType,
    width: i32,
    height: i32,
}

impl ImageBuilder {
    /// `ImageBuilder.builder()`.
    pub fn builder() -> ImageBuilder {
        ImageBuilder { image_type: ImageType::Png, width: DEFAULT_WIDTH, height: DEFAULT_HEIGHT }
    }

    /// `build()`.
    pub fn build(self) -> Base64ImageRuleConfig {
        Base64ImageRuleConfig::new(self.image_type, self.width, self.height)
    }

    /// `type(imageType)`; `None` (Java's `null`) fails with a `NullPointerException`.
    pub fn image_type(mut self, image_type: impl Into<Option<ImageType>>) -> Result<ImageBuilder> {
        self.image_type = image_type.into().ok_or_else(|| Error::new("NullPointerException", "Image type cannot be null"))?;
        Ok(self)
    }

    /// `width(width)`.
    pub fn width(mut self, width: i32) -> Result<ImageBuilder> {
        if width <= 0 {
            return Err(Error::illegal_argument("Width must be greater than 0"));
        }
        self.width = width;
        Ok(self)
    }

    /// `height(height)`.
    pub fn height(mut self, height: i32) -> Result<ImageBuilder> {
        if height <= 0 {
            return Err(Error::illegal_argument("Height must be greater than 0"));
        }
        self.height = height;
        Ok(self)
    }
}

/// Base64 data URLs of images: a grid of boxes, an eighth of the width wide, each of a random colour, in a raster
/// format through the JDK's `ImageIO` writers or as SVG.
#[derive(Clone, Copy)]
pub struct Image<'a> {
    faker: &'a Faker,
}

impl<'a> Image<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Image { faker }
    }

    /// `base64BMP()`.
    pub fn base64_bmp(&self) -> Result<String> {
        self.base64(ImageBuilder::builder().image_type(ImageType::Bmp)?.build())
    }

    /// `base64GIF()`.
    pub fn base64_gif(&self) -> Result<String> {
        self.base64(ImageBuilder::builder().image_type(ImageType::Gif)?.build())
    }

    /// `base64JPG()`: [`base64_jpeg`](Self::base64_jpeg).
    pub fn base64_jpg(&self) -> Result<String> {
        self.base64_jpeg()
    }

    /// `base64JPEG()`.
    pub fn base64_jpeg(&self) -> Result<String> {
        self.base64(ImageBuilder::builder().image_type(ImageType::Jpeg)?.build())
    }

    /// `base64PNG()`.
    pub fn base64_png(&self) -> Result<String> {
        self.base64(ImageBuilder::builder().image_type(ImageType::Png)?.build())
    }

    /// `base64SVG()`.
    pub fn base64_svg(&self) -> Result<String> {
        self.base64(ImageBuilder::builder().image_type(ImageType::Svg)?.build())
    }

    /// `base64TIFF()`.
    pub fn base64_tiff(&self) -> Result<String> {
        self.raster(ImageType::Tiff, DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    /// `base64(config)`.
    pub fn base64(&self, config: Base64ImageRuleConfig) -> Result<String> {
        if config.image_type == ImageType::Svg {
            Ok(self.vector(config.image_type, config.width, config.height))
        } else {
            self.raster(config.image_type, config.width, config.height)
        }
    }

    fn raster(&self, image_type: ImageType, width: i32, height: i32) -> Result<String> {
        if width <= 0 || height <= 0 {
            return Err(Error::illegal_argument(format!("Width ({width}) and height ({height}) cannot be <= 0")));
        }
        let (w, h) = (width as usize, height as usize);
        let mut image = BufferedImage::new(w, h, imageio::ImageType::IntRgb);
        let box_size = (w / 8).max(1);
        image.fill_rect(0, 0, w, h, [255, 255, 255]);
        for y in (0..h).step_by(box_size) {
            for x in (0..w).step_by(box_size) {
                let color = self.random_color();
                image.fill_rect(x, y, box_size, box_size, color);
            }
        }
        let bytes = imageio::write(&image, image_type.name()).ok_or_else(|| {
            Error::runtime(format!("Failed to generate image {} of size {width}x{height}", image_type.name()))
        })?;
        Ok(format!("data:{};base64,{}", image_type.mime_type(), crate::java::base64(&bytes)))
    }

    fn vector(&self, image_type: ImageType, width: i32, height: i32) -> String {
        let mut svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{DEFAULT_WIDTH}\" height=\"{DEFAULT_HEIGHT}\">");
        let box_size = (width / 8).max(1);
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                let [r, g, b] = self.random_color();
                svg.push_str(&format!(
                    "<rect x=\"{x}\" y=\"{y}\" width=\"{box_size}\" height=\"{box_size}\" fill=\"#{r:02x}{g:02x}{b:02x}\"/>"
                ));
                x += box_size;
            }
            y += box_size;
        }
        svg.push_str("</svg>");
        format!("data:{};base64,{}", image_type.mime_type(), crate::java::base64(svg.as_bytes()))
    }

    fn random_color(&self) -> [u8; 3] {
        let bytes = self.faker.random().next_random_bytes(3);
        [bytes[0] as u8, bytes[1] as u8, bytes[2] as u8]
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("base64BMP", &[], |f, _| s(Image::new(f).base64_bmp())),
    Method::new("base64GIF", &[], |f, _| s(Image::new(f).base64_gif())),
    Method::new("base64JPG", &[], |f, _| s(Image::new(f).base64_jpg())),
    Method::new("base64JPEG", &[], |f, _| s(Image::new(f).base64_jpeg())),
    Method::new("base64PNG", &[], |f, _| s(Image::new(f).base64_png())),
    Method::new("base64SVG", &[], |f, _| s(Image::new(f).base64_svg())),
    Method::new("base64TIFF", &[], |f, _| s(Image::new(f).base64_tiff())),
    // An expression cannot build a `Base64ImageRuleConfig` (`P::Opaque` never coerces); a template passes `null`.
    Method::new("base64", &[P::Opaque("Base64ImageRuleConfig")], |_, _| Err(crate::dispatch::npe("Cannot read field \"imageType\" because \"config\" is null")))
        .accepting_null(),
];
