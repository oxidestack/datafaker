//! Differential vectors of `Image` and `Fingerprint`: `tests/vectors/images.tsv` holds named cases recorded from
//! Datafaker 2.7.0 on JDK 25.0.2 by `scripts/Images.java` (`scripts/record.sh tests/vectors/images.tsv Images`); each
//! is rebuilt here on a fresh `Faker::with_seed(seed)` and must give the same data URL (or PNG bytes, as Base64),
//! compared in full up to 2000 characters and by length and SHA-256 above, then the same next `nextInt()`.

use datafaker::providers::{Base64ImageRuleConfig, FingerprintPatternType, ImageBuilder, ImageType};
use datafaker::{Error, Faker, Result};

fn image_type(name: &str) -> ImageType {
    ImageType::VALUES.into_iter().find(|t| t.name() == name).expect("an image type")
}

fn pattern(name: &str) -> FingerprintPatternType {
    FingerprintPatternType::VALUES.iter().copied().find(|p| p.constant() == name).expect("a pattern")
}

fn size(s: &str) -> (i32, i32) {
    let (w, h) = s.split_once('x').expect("<width>x<height>");
    (w.parse().unwrap(), h.parse().unwrap())
}

fn run(case: &str, faker: &Faker) -> Result<String> {
    let p: Vec<&str> = case.split('.').collect();
    match p[0] {
        "image" if p.len() == 2 => {
            let image = faker.image();
            match p[1] {
                "base64BMP" => image.base64_bmp(),
                "base64GIF" => image.base64_gif(),
                "base64JPG" => image.base64_jpg(),
                "base64JPEG" => image.base64_jpeg(),
                "base64PNG" => image.base64_png(),
                "base64SVG" => image.base64_svg(),
                "base64TIFF" => image.base64_tiff(),
                other => panic!("unknown case {other}"),
            }
        }
        "image" => {
            let (w, h) = size(p[2]);
            faker.image().base64(Base64ImageRuleConfig::new(image_type(p[1]), w, h))
        }
        "builder" => {
            let mut b = ImageBuilder::builder();
            if p[1] != "-" {
                b = b.image_type(if p[1] == "null" { None } else { Some(image_type(p[1])) })?;
            }
            if p[2] != "-" {
                b = b.width(p[2].parse().unwrap())?;
            }
            if p[3] != "-" {
                b = b.height(p[3].parse().unwrap())?;
            }
            faker.image().base64(b.build())
        }
        "fingerprint" => match p[1] {
            "png" => Ok(datafaker::java::base64(&faker.fingerprint().png()?)),
            "base64" => faker.fingerprint().base64(),
            pat => {
                let (w, h) = size(p[2]);
                faker.fingerprint().base64_with(w, h, pattern(pat))
            }
        },
        other => panic!("unknown case {other}"),
    }
}

fn show(s: &str) -> String {
    if s.encode_utf16().count() <= 2000 {
        return s.to_string();
    }
    let digest = datafaker::digests::sha256(s.as_bytes());
    format!("#{} {}", s.encode_utf16().count(), digest.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

#[test]
fn images_match_datafaker() {
    let text = include_str!("vectors/images.tsv");
    let mut failures = Vec::new();
    let mut rows = 0;
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let cols: Vec<&str> = line.splitn(3, '\t').collect();
        let seed: i64 = cols[0].parse().unwrap();
        let faker = Faker::with_seed(seed);
        let actual = match run(cols[1], &faker) {
            Ok(v) => format!("{} | next {}", show(&v), faker.random().next_int()),
            Err(e) => error(&e),
        };
        rows += 1;
        if actual != cols[2] {
            failures.push(format!("{} {}:\n  expected {}\n  actual   {}", cols[0], cols[1], cut(cols[2]), cut(&actual)));
        }
    }
    assert!(rows > 100, "{rows} rows");
    assert!(failures.is_empty(), "{} of {rows} rows differ:\n{}", failures.len(), failures.join("\n"));
}

fn error(e: &Error) -> String {
    format!("ERR {}: {}", e.simple_class(), e.message())
}

fn cut(s: &str) -> &str {
    &s[..s.len().min(220)]
}
