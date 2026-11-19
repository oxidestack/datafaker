//! `net.datafaker.providers.base.Fingerprint`, with `PatternType`.
//!
//! The ridges are computed with `f64::sin`, `exp`, `atan2` and `sqrt` where upstream calls `Math`'s: HotSpot's
//! intrinsics and the platform's `libm` may differ in the last bit, which moves a pixel only when a value lands
//! exactly on a ridge or rounding boundary; `tests/vectors/images.tsv` pins whole images.

use crate::dispatch::{JValue, Method, P};
use crate::java::imageio::{self, BufferedImage};
use crate::{Error, Faker, Result};

const DEFAULT_WIDTH: i32 = 200;
const DEFAULT_HEIGHT: i32 = 250;
const TWO_PI: f64 = 2.0 * std::f64::consts::PI;

provider_enum! {
    /// `Fingerprint.PatternType`: the three ridge-flow patterns.
    PatternType, "PatternType" {
        Whorl = "WHORL" => "",
        Loop = "LOOP" => "",
        Arch = "ARCH" => "",
    }
}

/// Synthetic fingerprint images (whorl, loop or arch ridges in an oval), as PNG.
#[derive(Clone, Copy)]
pub struct Fingerprint<'a> {
    faker: &'a Faker,
}

impl<'a> Fingerprint<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Fingerprint { faker }
    }

    /// `png()`: a 200 × 250 image of a random pattern.
    pub fn png(&self) -> Result<Vec<u8>> {
        let pattern = self.random_type()?;
        self.png_with(DEFAULT_WIDTH, DEFAULT_HEIGHT, pattern)
    }

    /// `png(width, height, pattern)`.
    pub fn png_with(&self, width: i32, height: i32, pattern: PatternType) -> Result<Vec<u8>> {
        self.png_of(width, height, Some(pattern))
    }

    /// `png(width, height, pattern)` for a pattern that may be `null`: the image is set up and its random core and
    /// minutiae drawn before the first pixel's `switch (patternType)` throws.
    fn png_of(&self, width: i32, height: i32, pattern: Option<PatternType>) -> Result<Vec<u8>> {
        let image = self.render(width, height, pattern)?;
        Ok(imageio::png::write(&image))
    }

    /// `base64()`: [`png`](Self::png) as a data URL.
    pub fn base64(&self) -> Result<String> {
        let pattern = self.random_type()?;
        self.base64_with(DEFAULT_WIDTH, DEFAULT_HEIGHT, pattern)
    }

    /// `base64(width, height, pattern)`.
    pub fn base64_with(&self, width: i32, height: i32, pattern: PatternType) -> Result<String> {
        self.base64_of(width, height, Some(pattern))
    }

    fn base64_of(&self, width: i32, height: i32, pattern: Option<PatternType>) -> Result<String> {
        Ok(format!("data:image/png;base64,{}", crate::java::base64(&self.png_of(width, height, pattern)?)))
    }

    fn random_type(&self) -> Result<PatternType> {
        Ok(PatternType::VALUES[self.faker.random().next_enum(PatternType::VALUES.len())?])
    }

    fn between(&self, min: f64, max: f64) -> f64 {
        self.faker.random().next_double_between(min, max)
    }

    fn render(&self, width: i32, height: i32, pattern: Option<PatternType>) -> Result<BufferedImage> {
        if width <= 0 || height <= 0 {
            return Err(Error::illegal_argument("w and h must be > 0"));
        }
        let mut image = BufferedImage::new(width as usize, height as usize, imageio::ImageType::ByteGray);
        let (w, h) = (width as f64, height as f64);
        let cx = w * (0.47 + self.between(-0.04, 0.04));
        let cy = h * (0.47 + self.between(-0.04, 0.04));
        let ridge_period = 11.0 + self.between(-2.0, 3.0);
        let ridge_fraction = 0.42 + self.between(-0.04, 0.06);
        let spiral_factor = 0.5 + self.between(0.0, 0.5);
        let loop_amplitude = h * (0.18 + self.between(0.0, 0.10));
        let loop_sigma = w.min(h) * (0.28 + self.between(0.0, 0.12));
        let arch_amplitude = h * (0.12 + self.between(0.0, 0.08));
        let arch_sigma = w * (0.30 + self.between(0.0, 0.15));

        const NUM_NOISE: usize = 3;
        let noise_amp = [3.2, 1.6, 0.7];
        let noise_freq = [0.014, 0.032, 0.068];
        let mut phases_x = [0.0; NUM_NOISE];
        let mut phases_y = [0.0; NUM_NOISE];
        for i in 0..NUM_NOISE {
            phases_x[i] = self.between(0.0, TWO_PI);
            phases_y[i] = self.between(0.0, TWO_PI);
        }

        let mask_a = w * 0.44;
        let mask_b = h * 0.46;
        let fade_zone = 0.13;

        let num_minutiae = 18 + self.faker.random().next_int_between(0, 18)? as usize;
        let minutia_sigma = ridge_period * 0.85;
        let two_sigma_min_sq = 2.0 * minutia_sigma * minutia_sigma;
        let minutia_cutoff_sq = 9.0 * minutia_sigma * minutia_sigma;
        let mut minutiae = Vec::with_capacity(num_minutiae);
        for _ in 0..num_minutiae {
            let (mut rx, mut ry);
            loop {
                rx = self.between(-0.40, 0.40);
                ry = self.between(-0.42, 0.42);
                if rx * rx / 0.16 + ry * ry / 0.18 <= 0.85 {
                    break;
                }
            }
            let sign = if self.faker.random().next_boolean() { 1.0 } else { -1.0 };
            minutiae.push([w * 0.5 + rx * w, h * 0.5 + ry * h, sign * ridge_period * 0.5]);
        }

        let ridge_gray = self.faker.random().next_int_between(25, 65)?;
        let valley_gray = self.faker.random().next_int_between(195, 235)?;

        for y in 0..height {
            for x in 0..width {
                let (xf, yf) = (x as f64, y as f64);
                let (mut noise_x, mut noise_y) = (0.0, 0.0);
                for i in 0..NUM_NOISE {
                    let f = noise_freq[i];
                    noise_x += noise_amp[i] * (xf * f + yf * f * 0.6 + phases_x[i]).sin();
                    noise_y += noise_amp[i] * (yf * f + xf * f * 0.6 + phases_y[i]).sin();
                }
                let px = xf + noise_x;
                let py = yf + noise_y;
                let dx = px - cx;
                let dy = py - cy;
                let r = (dx * dx + dy * dy).sqrt();
                let Some(pattern) = pattern else {
                    return Err(crate::dispatch::npe(
                        "Cannot invoke \"net.datafaker.providers.base.Fingerprint$PatternType.ordinal()\" because \"patternType\" is null",
                    ));
                };
                let mut ridge_val = match pattern {
                    PatternType::Whorl => {
                        let theta = dy.atan2(dx);
                        r + spiral_factor * ridge_period * theta / std::f64::consts::PI
                    }
                    PatternType::Loop => {
                        let pull = loop_amplitude * (-r * r / (2.0 * loop_sigma * loop_sigma)).exp();
                        let loop_dy = dy - pull;
                        (dx * dx + loop_dy * loop_dy).sqrt()
                    }
                    PatternType::Arch => {
                        let arch = arch_amplitude * (-dx * dx / (2.0 * arch_sigma * arch_sigma)).exp();
                        dy + arch
                    }
                };
                for m in &minutiae {
                    let dxm = px - m[0];
                    let dym = py - m[1];
                    let rm2 = dxm * dxm + dym * dym;
                    if rm2 < minutia_cutoff_sq {
                        ridge_val += m[2] * (-rm2 / two_sigma_min_sq).exp();
                    }
                }
                // Java's `%` on doubles is `fmod`, as Rust's.
                let modulo = ((ridge_val % ridge_period) + ridge_period) % ridge_period;
                let is_ridge = modulo < ridge_period * ridge_fraction;
                let mx = (xf - w * 0.5) / mask_a;
                let my = (yf - h * 0.5) / mask_b;
                let mask_dist = mx * mx + my * my;
                let pixel = if mask_dist >= 1.0 {
                    255
                } else {
                    let fade = ((1.0 - mask_dist) / fade_zone).min(1.0);
                    let base = if is_ridge { ridge_gray } else { valley_gray } as f64;
                    java_round(base * fade + 255.0 * (1.0 - fade)) as i32
                };
                // `setSample` on a byte raster keeps the low eight bits.
                image.set_gray(x as usize, y as usize, pixel as u8);
            }
        }
        Ok(image)
    }
}

/// `Math.round(double)`: the nearest `long`, ties toward positive infinity.
fn java_round(v: f64) -> i64 {
    let floor = v.floor();
    if v - floor >= 0.5 { floor as i64 + 1 } else { floor as i64 }
}

pub(crate) static METHODS: &[Method] = &[
    // `byte[]`: the bytes as a list of `byte` values (an array's `toString()` is an identity hash upstream).
    Method::new("png", &[], |f, _| Ok(bytes(Fingerprint::new(f).png()?))),
    Method::new("png", &[P::Int, P::Int, PatternType::PARAM], |f, a| {
        Ok(bytes(Fingerprint::new(f).png_of(a[0].as_int(), a[1].as_int(), PatternType::of_nullable(&a[2]))?))
    })
    .accepting_null(),
    Method::new("base64", &[], |f, _| Fingerprint::new(f).base64().map(JValue::Str)),
    Method::new("base64", &[P::Int, P::Int, PatternType::PARAM], |f, a| {
        Fingerprint::new(f).base64_of(a[0].as_int(), a[1].as_int(), PatternType::of_nullable(&a[2])).map(JValue::Str)
    })
    .accepting_null(),
];

fn bytes(b: Vec<u8>) -> JValue {
    JValue::List(b.into_iter().map(|b| JValue::Int(b as i8 as i32)).collect())
}
