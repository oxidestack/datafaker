//! Ports of Datafaker 2.7.0's `ImageTest` and `FingerprintTest`, with `assertions/ImageAssert` (`assertThatImage`).
//! Where upstream decodes an image with `ImageIO.read` to check its size, the port reads the size from the format's
//! own header (BMP info header, GIF logical screen, PNG `IHDR`, JPEG `SOF0`, TIFF IFD); `tests/vectors_images.rs`
//! checks the bytes themselves against the JVM. Fakers are unseeded as upstream, except where a test seeds one.

use datafaker::providers::{Base64ImageRuleConfig, FingerprintPatternType, ImageBuilder, ImageType};
use datafaker::Faker;

fn decode_base64(s: &str) -> Option<Vec<u8>> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    };
    let bytes = s.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let pad = chunk.iter().rev().take_while(|&&c| c == b'=').count();
        let mut n = 0u32;
        for &c in &chunk[..4 - pad] {
            n = n << 6 | value(c)? as u32;
        }
        n <<= 6 * pad as u32;
        out.extend_from_slice(&n.to_be_bytes()[1..4 - pad]);
    }
    Some(out)
}

fn be16(b: &[u8], at: usize) -> usize {
    u16::from_be_bytes([b[at], b[at + 1]]) as usize
}

fn be32(b: &[u8], at: usize) -> usize {
    u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]) as usize
}

fn le16(b: &[u8], at: usize) -> usize {
    u16::from_le_bytes([b[at], b[at + 1]]) as usize
}

fn le32(b: &[u8], at: usize) -> usize {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]) as usize
}

/// What `ImageIO.read(bytes)` reports as width and height, read from the header of a format it knows.
fn image_size(b: &[u8]) -> (usize, usize) {
    if b.starts_with(b"BM") {
        (le32(b, 18), le32(b, 22))
    } else if b.starts_with(b"GIF89a") || b.starts_with(b"GIF87a") {
        (le16(b, 6), le16(b, 8))
    } else if b.starts_with(&[0x89, b'P', b'N', b'G', 13, 10, 26, 10]) {
        assert_eq!(&b[12..16], b"IHDR");
        (be32(b, 16), be32(b, 20))
    } else if b.starts_with(&[0xFF, 0xD8]) {
        let mut at = 2;
        loop {
            assert_eq!(b[at], 0xFF, "a JPEG marker at {at}");
            let length = be16(b, at + 2);
            if b[at + 1] == 0xC0 {
                return (be16(b, at + 7), be16(b, at + 5));
            }
            at += 2 + length;
        }
    } else if b.starts_with(b"MM\0\x2a") {
        let ifd = be32(b, 4);
        let (mut width, mut height) = (0, 0);
        for i in 0..be16(b, ifd) {
            let e = ifd + 2 + 12 * i;
            let value = if be16(b, e + 2) == 3 { be16(b, e + 8) } else { be32(b, e + 8) };
            match be16(b, e) {
                256 => width = value,
                257 => height = value,
                _ => {}
            }
        }
        (width, height)
    } else {
        panic!("not an image ImageIO reads: {:?}", &b[..b.len().min(8)]);
    }
}

/// `ImageAssert.is(type, width, height)`.
fn assert_that_image(url: &str, image_type: ImageType, width: usize, height: usize) {
    let prefix = format!("data:{};base64,", image_type.mime_type());
    assert!(url.starts_with(&prefix), "Expected image data URL starting with \"{prefix}\", but received: \"{url}\"");
    let content = &url[url.find(',').unwrap() + 1..];
    assert!(!content.trim().is_empty());
    let bytes = decode_base64(content).expect("Base64");
    if image_type == ImageType::Svg {
        let xml = String::from_utf8(bytes).unwrap();
        let re = regex::Regex::new(r#"^<svg xmlns=".+" width="256" height="256">.*</svg>$"#).unwrap();
        assert!(re.is_match(&xml), "{xml}");
        // The document's one child is `svg`, whose children are the boxes.
        let body = &xml[xml.find('>').unwrap() + 1..xml.len() - "</svg>".len()];
        let rects = regex::Regex::new(r##"<rect x="\d+" y="\d+" width="\d+" height="\d+" fill="#[0-9a-f]{6}"/>"##).unwrap();
        assert_eq!(rects.replace_all(body, ""), "", "only rect children");
        let box_size = width / 8;
        assert_eq!(rects.find_iter(body).count(), (width / box_size) * (height / box_size));
    } else {
        assert_eq!(image_size(&bytes), (width, height));
    }
}

fn assert_image(url: &str, image_type: ImageType) {
    assert_that_image(url, image_type, 256, 256);
}

// ImageTest

#[test]
fn bmp() {
    assert_image(&Faker::new().image().base64_bmp().unwrap(), ImageType::Bmp);
}

#[test]
fn gif() {
    assert_image(&Faker::new().image().base64_gif().unwrap(), ImageType::Gif);
}

#[test]
fn png() {
    assert_image(&Faker::new().image().base64_png().unwrap(), ImageType::Png);
}

#[test]
fn jpg() {
    assert_image(&Faker::new().image().base64_jpg().unwrap(), ImageType::Jpeg);
}

#[test]
fn jpeg() {
    assert_image(&Faker::new().image().base64_jpeg().unwrap(), ImageType::Jpeg);
}

#[test]
fn svg() {
    assert_image(&Faker::new().image().base64_svg().unwrap(), ImageType::Svg);
}

#[test]
fn tiff() {
    assert_image(&Faker::new().image().base64_tiff().unwrap(), ImageType::Tiff);
}

#[test]
fn base64() {
    let faker = Faker::new();
    for image_type in ImageType::VALUES {
        let image = faker.image().base64(Base64ImageRuleConfig::new(image_type, 1000, 1000)).unwrap();
        assert_that_image(&image, image_type, 1000, 1000);
    }
}

#[test]
fn default_builder_generates_png_image() {
    let image = Faker::new().image().base64(ImageBuilder::builder().build()).unwrap();
    assert_image(&image, ImageType::Png);
}

#[test]
fn custom_base64_builder() {
    let gif = Faker::new().image().base64(ImageBuilder::builder().image_type(ImageType::Gif).unwrap().build()).unwrap();
    assert_image(&gif, ImageType::Gif);
}

#[test]
fn tiny_base64_builder() {
    let builder = ImageBuilder::builder().height(1).unwrap().width(1).unwrap().image_type(ImageType::Png).unwrap();
    let tiny = Faker::new().image().base64(builder.build()).unwrap();
    assert_that_image(&tiny, ImageType::Png, 1, 1);
}

#[test]
fn large_base64_builder() {
    let builder = ImageBuilder::builder().height(1000).unwrap().width(2000).unwrap().image_type(ImageType::Bmp).unwrap();
    let large = Faker::new().image().base64(builder.build()).unwrap();
    assert_that_image(&large, ImageType::Bmp, 2000, 1000);
}

#[test]
fn should_error_on_illegal_type() {
    let e = ImageBuilder::builder().image_type(None).unwrap_err();
    assert_eq!(e.simple_class(), "NullPointerException");
}

#[test]
fn should_error_on_negative_width() {
    assert_eq!(ImageBuilder::builder().width(-1).unwrap_err().simple_class(), "IllegalArgumentException");
}

#[test]
fn should_error_on_zero_width() {
    assert_eq!(ImageBuilder::builder().width(0).unwrap_err().simple_class(), "IllegalArgumentException");
}

#[test]
fn should_error_on_negative_height() {
    assert_eq!(ImageBuilder::builder().height(-1).unwrap_err().simple_class(), "IllegalArgumentException");
}

#[test]
fn should_error_on_zero_height() {
    assert_eq!(ImageBuilder::builder().height(0).unwrap_err().simple_class(), "IllegalArgumentException");
}

// FingerprintTest

const DATA_URL: &str = "data:image/png;base64,";

#[test]
fn png_returns_png_bytes() {
    let bytes = Faker::new().fingerprint().png().unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn png_has_expected_dimensions() {
    let bytes = Faker::new().fingerprint().png().unwrap();
    assert_eq!(image_size(&bytes), (200, 250));
}

#[test]
fn base64_returns_data_url() {
    let url = Faker::new().fingerprint().base64().unwrap();
    assert!(url.starts_with(DATA_URL));
    let encoded = &url[DATA_URL.len()..];
    assert!(!encoded.trim().is_empty());
    assert!(decode_base64(encoded).is_some());
}

#[test]
fn base64_decodes_back_to_png() {
    let url = Faker::new().fingerprint().base64().unwrap();
    let bytes = decode_base64(&url[DATA_URL.len()..]).unwrap();
    assert_eq!(image_size(&bytes), (200, 250));
}

/// Upstream also writes each image to `target/surefire-reports/fingerprint-<PATTERN>.png` for a look; not ported.
#[test]
fn each_pattern_type_produces_valid_png() {
    for &pattern in FingerprintPatternType::VALUES {
        let bytes = Faker::new().fingerprint().png_with(400, 500, pattern).unwrap();
        assert_eq!(&bytes[..2], &[0x89, b'P']);
        assert_eq!(image_size(&bytes), (400, 500));
    }
}

#[test]
fn two_png_calls_produce_different_images() {
    let faker = Faker::new();
    let first = faker.fingerprint().png().unwrap();
    let second = faker.fingerprint().png().unwrap();
    assert_ne!(first, second);
}

#[test]
fn two_png_calls_with_same_see_produce_same_images() {
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as i64;
    let first = Faker::with_seed(seed).fingerprint().png().unwrap();
    let second = Faker::with_seed(seed).fingerprint().png().unwrap();
    assert_eq!(first, second);
}
