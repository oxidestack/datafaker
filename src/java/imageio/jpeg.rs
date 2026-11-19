//! `com.sun.imageio.plugins.jpeg.JPEGImageWriter` with no write parameters, for `TYPE_INT_RGB` images: the JDK hands
//! the image to its bundled IJG libjpeg 6b (`libjavajpeg`), configured by `imageioJPEG.c` for the default metadata,
//! and writes the JFIF 1.02 `APP0` marker itself. This is that libjpeg path, ported: RGB to YCbCr (`jccolor.c`,
//! fixed point), Y at full size and Cb, Cr averaged over 2 × 2 (`jcsample.c`, bias 1, 2, 1, …; edges replicated by
//! `expand_right_edge` and `jcprepct.c`), the integer DCT (`jfdctint.c`, `jcdctmgr.c`) with the JDK's default
//! tables (`JPEGQTable.K1Div2Luminance`, `K2Div2Chrominance`, the standard Huffman tables), dummy blocks at the
//! right and bottom edges of an MCU (`jccoefct.c`) and Huffman coding with byte stuffing (`jchuff.c`).

use super::BufferedImage;

const K1_DIV2: [u16; 64] = [
    8, 6, 5, 8, 12, 20, 26, 31, 6, 6, 7, 10, 13, 29, 30, 28, 7, 7, 8, 12, 20, 29, 35, 28, 7, 9, 11, 15, 26, 44, 40, 31, 9, 11, 19, 28, 34, 55,
    52, 39, 12, 18, 28, 32, 41, 52, 57, 46, 25, 32, 39, 44, 52, 61, 60, 51, 36, 46, 48, 49, 56, 50, 52, 50,
];
const K2_DIV2: [u16; 64] = [
    9, 9, 12, 24, 50, 50, 50, 50, 9, 11, 13, 33, 50, 50, 50, 50, 12, 13, 28, 50, 50, 50, 50, 50, 24, 33, 50, 50, 50, 50, 50, 50, 50, 50, 50,
    50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50,
];

/// `jpeg_natural_order`: the natural-order index of each zigzag position.
const NATURAL_ORDER: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57,
    50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

const BITS_DC_LUMINANCE: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const VAL_DC: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const BITS_DC_CHROMINANCE: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
const BITS_AC_LUMINANCE: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
const VAL_AC_LUMINANCE: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1,
    0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26,
    0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56,
    0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85,
    0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa,
    0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6,
    0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9,
    0xfa,
];
const BITS_AC_CHROMINANCE: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77];
const VAL_AC_CHROMINANCE: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42,
    0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0, 0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19,
    0x1a, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55,
    0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x82, 0x83,
    0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8,
    0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4,
    0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9,
    0xfa,
];

/// `jpeg_make_c_derived_tbl`: each symbol's code and code length.
struct Huffman {
    code: [u32; 256],
    size: [u32; 256],
}

impl Huffman {
    fn new(bits: &[u8; 16], vals: &[u8]) -> Huffman {
        let mut t = Huffman { code: [0; 256], size: [0; 256] };
        let mut code = 0u32;
        let mut k = 0;
        for (l, &n) in bits.iter().enumerate() {
            for _ in 0..n {
                t.code[vals[k] as usize] = code;
                t.size[vals[k] as usize] = l as u32 + 1;
                code += 1;
                k += 1;
            }
            code <<= 1;
        }
        t
    }
}

/// `jchuff.c`'s bit buffer.
struct BitWriter {
    out: Vec<u8>,
    put_buffer: i64,
    put_bits: i32,
}

impl BitWriter {
    fn emit(&mut self, code: u32, size: u32) {
        let mut put_buffer = (code as i64) & ((1i64 << size) - 1);
        let mut put_bits = self.put_bits + size as i32;
        put_buffer <<= 24 - put_bits;
        put_buffer |= self.put_buffer;
        while put_bits >= 8 {
            let c = ((put_buffer >> 16) & 0xFF) as u8;
            self.out.push(c);
            if c == 0xFF {
                self.out.push(0);
            }
            put_buffer <<= 8;
            put_bits -= 8;
        }
        self.put_buffer = put_buffer & 0xFF_FFFF;
        self.put_bits = put_bits;
    }

    fn flush(&mut self) {
        self.emit(0x7F, 7);
        self.put_buffer = 0;
        self.put_bits = 0;
    }
}

const CONST_BITS: i32 = 13;
const PASS1_BITS: i32 = 2;

fn descale(x: i64, n: i32) -> i64 {
    (x + (1 << (n - 1))) >> n
}

/// `jpeg_fdct_islow`.
fn fdct_islow(data: &mut [i64; 64]) {
    for pass in 0..2 {
        for ctr in 0..8 {
            let (base, step) = if pass == 0 { (ctr * 8, 1) } else { (ctr, 8) };
            let d = |i: usize| base + i * step;
            let tmp0 = data[d(0)] + data[d(7)];
            let tmp7 = data[d(0)] - data[d(7)];
            let tmp1 = data[d(1)] + data[d(6)];
            let tmp6 = data[d(1)] - data[d(6)];
            let tmp2 = data[d(2)] + data[d(5)];
            let tmp5 = data[d(2)] - data[d(5)];
            let tmp3 = data[d(3)] + data[d(4)];
            let tmp4 = data[d(3)] - data[d(4)];
            let tmp10 = tmp0 + tmp3;
            let tmp13 = tmp0 - tmp3;
            let tmp11 = tmp1 + tmp2;
            let tmp12 = tmp1 - tmp2;
            let (shift, scale) = if pass == 0 { (CONST_BITS - PASS1_BITS, 0) } else { (CONST_BITS + PASS1_BITS, PASS1_BITS) };
            if pass == 0 {
                data[d(0)] = (tmp10 + tmp11) << PASS1_BITS;
                data[d(4)] = (tmp10 - tmp11) << PASS1_BITS;
            } else {
                data[d(0)] = descale(tmp10 + tmp11, scale);
                data[d(4)] = descale(tmp10 - tmp11, scale);
            }
            let z1 = (tmp12 + tmp13) * 4433;
            data[d(2)] = descale(z1 + tmp13 * 6270, shift);
            data[d(6)] = descale(z1 + tmp12 * -15137, shift);
            let z1 = tmp4 + tmp7;
            let z2 = tmp5 + tmp6;
            let z3 = tmp4 + tmp6;
            let z4 = tmp5 + tmp7;
            let z5 = (z3 + z4) * 9633;
            let tmp4 = tmp4 * 2446;
            let tmp5 = tmp5 * 16819;
            let tmp6 = tmp6 * 25172;
            let tmp7 = tmp7 * 12299;
            let z1 = z1 * -7373;
            let z2 = z2 * -20995;
            let z3 = z3 * -16069 + z5;
            let z4 = z4 * -3196 + z5;
            data[d(7)] = descale(tmp4 + z1 + z3, shift);
            data[d(5)] = descale(tmp5 + z2 + z4, shift);
            data[d(3)] = descale(tmp6 + z2 + z3, shift);
            data[d(1)] = descale(tmp7 + z1 + z4, shift);
        }
    }
}

/// A component's padded sample plane.
struct Plane {
    width: usize,
    samples: Vec<u8>,
}

impl Plane {
    fn at(&self, x: usize, y: usize) -> u8 {
        self.samples[y * self.width + x]
    }
}

/// `forward_DCT`: the quantized coefficients of the block at `(x, y)`, in natural order.
fn forward_dct(plane: &Plane, x: usize, y: usize, qtable: &[u16; 64]) -> [i32; 64] {
    let mut ws = [0i64; 64];
    for r in 0..8 {
        for c in 0..8 {
            ws[r * 8 + c] = plane.at(x + c, y + r) as i64 - 128;
        }
    }
    fdct_islow(&mut ws);
    let mut out = [0i32; 64];
    for i in 0..64 {
        let qval = (qtable[i] as i64) << 3;
        let mut temp = ws[i];
        let negative = temp < 0;
        if negative {
            temp = -temp;
        }
        temp += qval >> 1;
        temp = if temp >= qval { temp / qval } else { 0 };
        out[i] = if negative { -temp } else { temp } as i32;
    }
    out
}

fn encode_block(w: &mut BitWriter, block: &[i32; 64], last_dc: &mut i32, dc: &Huffman, ac: &Huffman) {
    let mut temp = block[0] - *last_dc;
    *last_dc = block[0];
    let mut temp2 = temp;
    if temp < 0 {
        temp = -temp;
        temp2 -= 1;
    }
    let nbits = 32 - (temp as u32).leading_zeros();
    w.emit(dc.code[nbits as usize], dc.size[nbits as usize]);
    if nbits > 0 {
        w.emit(temp2 as u32, nbits);
    }
    let mut r = 0;
    for &k in &NATURAL_ORDER[1..] {
        let mut temp = block[k];
        if temp == 0 {
            r += 1;
            continue;
        }
        while r > 15 {
            w.emit(ac.code[0xF0], ac.size[0xF0]);
            r -= 16;
        }
        let mut temp2 = temp;
        if temp < 0 {
            temp = -temp;
            temp2 -= 1;
        }
        let nbits = 32 - (temp as u32).leading_zeros();
        let i = (r << 4) + nbits as usize;
        w.emit(ac.code[i], ac.size[i]);
        w.emit(temp2 as u32, nbits);
        r = 0;
    }
    if r > 0 {
        w.emit(ac.code[0], ac.size[0]);
    }
}

fn marker(out: &mut Vec<u8>, code: u8, body: &[u8]) {
    out.extend_from_slice(&[0xFF, code]);
    out.extend_from_slice(&((body.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(body);
}

fn dqt(out: &mut Vec<u8>, id: u8, table: &[u16; 64]) {
    let mut body = vec![id];
    body.extend(NATURAL_ORDER.iter().map(|&i| table[i] as u8));
    marker(out, 0xDB, &body);
}

fn dht(out: &mut Vec<u8>, class_id: u8, bits: &[u8; 16], vals: &[u8]) {
    let mut body = vec![class_id];
    body.extend_from_slice(bits);
    body.extend_from_slice(vals);
    marker(out, 0xC4, &body);
}

/// `ImageIO.write(image, "jpeg", stream)`.
pub fn write(image: &BufferedImage) -> Vec<u8> {
    let (w, h) = (image.width(), image.height());
    let mut out = vec![0xFF, 0xD8];
    marker(&mut out, 0xE0, b"JFIF\0\x01\x02\0\0\x01\0\x01\0\0");
    dqt(&mut out, 0, &K1_DIV2);
    dqt(&mut out, 1, &K2_DIV2);
    let mut sof = vec![8];
    sof.extend_from_slice(&(h as u16).to_be_bytes());
    sof.extend_from_slice(&(w as u16).to_be_bytes());
    sof.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
    marker(&mut out, 0xC0, &sof);
    dht(&mut out, 0x00, &BITS_DC_LUMINANCE, &VAL_DC);
    dht(&mut out, 0x10, &BITS_AC_LUMINANCE, &VAL_AC_LUMINANCE);
    dht(&mut out, 0x01, &BITS_DC_CHROMINANCE, &VAL_DC);
    dht(&mut out, 0x11, &BITS_AC_CHROMINANCE, &VAL_AC_CHROMINANCE);
    marker(&mut out, 0xDA, &[3, 1, 0x00, 2, 0x11, 3, 0x11, 0, 63, 0]);

    // Colour conversion (`rgb_ycc_convert`), the rows in pairs, an odd last row repeated (`expand_bottom_edge`).
    const SCALEBITS: i64 = 16;
    let fix = |x: f64| (x * (1i64 << SCALEBITS) as f64 + 0.5) as i64;
    let one_half = 1i64 << (SCALEBITS - 1);
    let cbcr_offset = 128i64 << SCALEBITS;
    let h2 = h.div_ceil(2) * 2;
    let mut ycc = [vec![0u8; w * h2], vec![0u8; w * h2], vec![0u8; w * h2]];
    for y in 0..h2 {
        let src = y.min(h - 1);
        for x in 0..w {
            let p = image.pixel(x, src);
            let (r, g, b) = (p[0] as i64, p[1] as i64, p[2] as i64);
            ycc[0][y * w + x] = ((fix(0.29900) * r + fix(0.58700) * g + fix(0.11400) * b + one_half) >> SCALEBITS) as u8;
            ycc[1][y * w + x] = ((-fix(0.16874) * r + -fix(0.33126) * g + fix(0.50000) * b + cbcr_offset + one_half - 1) >> SCALEBITS) as u8;
            ycc[2][y * w + x] = ((fix(0.50000) * r + cbcr_offset + one_half - 1 + -fix(0.41869) * g + -fix(0.08131) * b) >> SCALEBITS) as u8;
        }
    }

    // Geometry (`initial_setup`, `per_scan_setup`): Y is 2 × 2 blocks per MCU, Cb and Cr one block each.
    let y_wib = w.div_ceil(8);
    let y_hib = h.div_ceil(8);
    let c_wib = w.div_ceil(16);
    let mcus_per_row = w.div_ceil(16);
    let imcu_rows = h.div_ceil(16);

    // Y: full size, right edge replicated to whole blocks, bottom replicated to whole iMCU rows.
    let y_plane = {
        let width = y_wib * 8;
        let rows = imcu_rows * 16;
        let mut s = vec![0u8; width * rows];
        for y in 0..rows {
            let src = y.min(h2 - 1);
            for x in 0..width {
                s[y * width + x] = ycc[0][src * w + x.min(w - 1)];
            }
        }
        Plane { width, samples: s }
    };
    // Cb, Cr: `h2v2_downsample` over the rows widened to twice the output columns.
    let chroma = |c: &Vec<u8>| {
        let width = c_wib * 8;
        let rows = imcu_rows * 8;
        let real = h2 / 2;
        let mut s = vec![0u8; width * rows];
        let at = |x: usize, y: usize| c[y * w + x.min(w - 1)] as u32;
        for oy in 0..real {
            let mut bias = 1;
            for ox in 0..width {
                let v = (at(2 * ox, 2 * oy) + at(2 * ox + 1, 2 * oy) + at(2 * ox, 2 * oy + 1) + at(2 * ox + 1, 2 * oy + 1) + bias) >> 2;
                s[oy * width + ox] = v as u8;
                bias ^= 3;
            }
        }
        for oy in real..rows {
            s.copy_within((real - 1) * width..real * width, oy * width);
        }
        Plane { width, samples: s }
    };
    let cb = chroma(&ycc[1]);
    let cr = chroma(&ycc[2]);

    let dc_l = Huffman::new(&BITS_DC_LUMINANCE, &VAL_DC);
    let ac_l = Huffman::new(&BITS_AC_LUMINANCE, &VAL_AC_LUMINANCE);
    let dc_c = Huffman::new(&BITS_DC_CHROMINANCE, &VAL_DC);
    let ac_c = Huffman::new(&BITS_AC_CHROMINANCE, &VAL_AC_CHROMINANCE);
    let mut bits = BitWriter { out: Vec::new(), put_buffer: 0, put_bits: 0 };
    let mut last_dc = [0i32; 3];
    let y_last_col_width = if y_wib % 2 == 0 { 2 } else { 1 };
    let y_last_row_height = if y_hib % 2 == 0 { 2 } else { 1 };
    for row in 0..imcu_rows {
        for col in 0..mcus_per_row {
            let blockcnt = if col < mcus_per_row - 1 { 2 } else { y_last_col_width };
            let mut blocks: Vec<[i32; 64]> = Vec::with_capacity(4);
            for yindex in 0..2 {
                if row < imcu_rows - 1 || yindex < y_last_row_height {
                    for bi in 0..2 {
                        if bi < blockcnt {
                            blocks.push(forward_dct(&y_plane, col * 16 + bi * 8, row * 16 + yindex * 8, &K1_DIV2));
                        } else {
                            let mut dummy = [0i32; 64];
                            dummy[0] = blocks[blocks.len() - 1][0];
                            blocks.push(dummy);
                        }
                    }
                } else {
                    let mut dummy = [0i32; 64];
                    dummy[0] = blocks[blocks.len() - 1][0];
                    blocks.push(dummy);
                    blocks.push(dummy);
                }
            }
            for b in &blocks {
                encode_block(&mut bits, b, &mut last_dc[0], &dc_l, &ac_l);
            }
            let b = forward_dct(&cb, col * 8, row * 8, &K2_DIV2);
            encode_block(&mut bits, &b, &mut last_dc[1], &dc_c, &ac_c);
            let b = forward_dct(&cr, col * 8, row * 8, &K2_DIV2);
            encode_block(&mut bits, &b, &mut last_dc[2], &dc_c, &ac_c);
        }
    }
    bits.flush();
    out.extend_from_slice(&bits.out);
    out.extend_from_slice(&[0xFF, 0xD9]);
    out
}
