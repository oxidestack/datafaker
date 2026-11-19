//! `com.sun.imageio.plugins.gif.GIFImageWriter` with no write parameters, for `TYPE_INT_RGB` images: the image is
//! first reduced to at most 256 colours by `PaletteBuilder.createIndexedImage` (an octree, quirks included), then
//! written as GIF89a with that palette as the global colour table (padded to a power of two with its first entry),
//! one interlaced image, 8-bit LZW codes (`LZWCompressor`, `LZWStringTable`, `BitFile`) in 255-byte blocks.

use super::BufferedImage;

const MAXLEVEL: usize = 8;

/// `PaletteBuilder.ColorNode`, in an arena.
#[derive(Default, Clone)]
struct ColorNode {
    is_leaf: bool,
    child_count: u32,
    children: [Option<usize>; 8],
    color_count: i64,
    red: i64,
    green: i64,
    blue: i64,
    palette_index: usize,
    level: usize,
    next_reducible: Option<usize>,
}

/// `com.sun.imageio.plugins.common.PaletteBuilder` for an opaque image.
struct PaletteBuilder {
    nodes: Vec<ColorNode>,
    root: Option<usize>,
    reduce_list: [Option<usize>; MAXLEVEL + 1],
    curr_size: usize,
    required_size: usize,
}

impl PaletteBuilder {
    fn branch_index(rgb: [u8; 3], level: usize) -> usize {
        let shift = MAXLEVEL - level;
        let bit = |v: u8| ((v as usize) >> shift) & 1;
        (bit(rgb[0]) << 2) | (bit(rgb[1]) << 1) | bit(rgb[2])
    }

    fn insert_node(&mut self, node: Option<usize>, rgb: [u8; 3], level: usize) -> usize {
        let id = match node {
            Some(id) => id,
            None => {
                let leaf = level > MAXLEVEL;
                if leaf {
                    self.curr_size += 1;
                }
                self.nodes.push(ColorNode { is_leaf: leaf, level, ..ColorNode::default() });
                self.nodes.len() - 1
            }
        };
        let n = &mut self.nodes[id];
        n.color_count += 1;
        n.red += rgb[0] as i64;
        n.green += rgb[1] as i64;
        n.blue += rgb[2] as i64;
        if !n.is_leaf {
            let branch = Self::branch_index(rgb, level);
            let child = n.children[branch];
            if child.is_none() {
                n.child_count += 1;
                if n.child_count == 2 {
                    n.next_reducible = self.reduce_list[level];
                    self.reduce_list[level] = Some(id);
                }
            }
            let child = self.insert_node(child, rgb, level + 1);
            self.nodes[id].children[branch] = Some(child);
        }
        id
    }

    fn leaf_child_count(&self, id: usize) -> usize {
        let n = &self.nodes[id];
        if n.is_leaf {
            return 0;
        }
        n.children.iter().flatten().map(|&c| if self.nodes[c].is_leaf { 1 } else { self.leaf_child_count(c) }).sum()
    }

    fn reduce_tree(&mut self) {
        let mut level = self.reduce_list.len() - 1;
        while self.reduce_list[level].is_none() {
            if level == 0 {
                return;
            }
            level -= 1;
        }
        let Some(mut this) = self.reduce_list[level] else { return };
        let mut p = this;
        let mut min_color_count = self.nodes[p].color_count;
        // Upstream's search: it compares with the next node but keeps `p` (and `p`'s count), so the node reduced is
        // the one after the chosen `this`, or the list's head.
        while let Some(next) = self.nodes[p].next_reducible {
            if min_color_count > self.nodes[next].color_count {
                this = p;
                min_color_count = self.nodes[p].color_count;
            }
            p = next;
        }
        if Some(this) == self.reduce_list[level] {
            self.reduce_list[level] = self.nodes[this].next_reducible;
        } else {
            let p = self.nodes[this].next_reducible.expect("a next node");
            self.nodes[this].next_reducible = self.nodes[p].next_reducible;
            this = p;
        }
        if self.nodes[this].is_leaf {
            return;
        }
        let leaf_child_count = self.leaf_child_count(this);
        self.nodes[this].is_leaf = true;
        self.curr_size = self.curr_size + 1 - leaf_child_count;
        self.nodes[this].children = [None; 8];
        self.nodes[this].child_count = 0;
    }

    fn find_palette_entry(&mut self, id: usize, mut index: usize, palette: &mut Vec<[u8; 3]>) -> usize {
        if self.nodes[id].is_leaf {
            let n = &mut self.nodes[id];
            palette[index] = [(n.red / n.color_count) as u8, (n.green / n.color_count) as u8, (n.blue / n.color_count) as u8];
            n.palette_index = index;
            index += 1;
        } else {
            for i in 0..8 {
                if let Some(c) = self.nodes[id].children[i] {
                    index = self.find_palette_entry(c, index, palette);
                }
            }
        }
        index
    }

    fn find_color_index(&self, id: usize, rgb: [u8; 3]) -> usize {
        let n = &self.nodes[id];
        if n.is_leaf {
            n.palette_index
        } else {
            self.find_color_index(n.children[Self::branch_index(rgb, n.level)].expect("a child on the colour's path"), rgb)
        }
    }

    /// `createIndexedImage(image)`: the palette and each pixel's index into it.
    fn create_indexed_image(image: &BufferedImage) -> (Vec<[u8; 3]>, Vec<u8>) {
        let mut pb = PaletteBuilder { nodes: Vec::new(), root: None, reduce_list: [None; MAXLEVEL + 1], curr_size: 0, required_size: 256 };
        let (w, h) = (image.width(), image.height());
        let rgb = |x: usize, y: usize| {
            let p = image.pixel(x, y);
            [p[0], p[1], p[2]]
        };
        for y in 0..h {
            for x in 0..w {
                let color = rgb(w - x - 1, h - y - 1);
                pb.root = Some(pb.insert_node(pb.root, color, 0));
                if pb.curr_size > pb.required_size {
                    pb.reduce_tree();
                }
            }
        }
        let mut palette = vec![[0u8; 3]; pb.curr_size];
        let root = pb.root.expect("a non-empty image");
        pb.find_palette_entry(root, 0, &mut palette);
        let mut indices = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                indices.push(pb.find_color_index(root, rgb(x, y)) as u8);
            }
        }
        (palette, indices)
    }
}

/// `BitFile` in GIF mode: codes packed from the low bit, in sub-blocks of at most 255 bytes.
struct BitFile {
    out: Vec<u8>,
    buffer: [u8; 256],
    index: usize,
    bits_left: u32,
}

impl BitFile {
    fn write_bits(&mut self, mut bits: u32, mut numbits: u32) {
        loop {
            if (self.index == 254 && self.bits_left == 0) || self.index > 254 {
                self.out.push(255);
                self.out.extend_from_slice(&self.buffer[..255]);
                self.buffer[0] = 0;
                self.index = 0;
                self.bits_left = 8;
            }
            if numbits <= self.bits_left {
                self.buffer[self.index] |= ((bits & ((1 << numbits) - 1)) << (8 - self.bits_left)) as u8;
                self.bits_left -= numbits;
                numbits = 0;
            } else {
                self.buffer[self.index] |= ((bits & ((1 << self.bits_left) - 1)) << (8 - self.bits_left)) as u8;
                bits >>= self.bits_left;
                numbits -= self.bits_left;
                self.index += 1;
                self.buffer[self.index] = 0;
                self.bits_left = 8;
            }
            if numbits == 0 {
                break;
            }
        }
    }

    fn flush(&mut self) {
        let n = self.index + if self.bits_left == 8 { 0 } else { 1 };
        if n > 0 {
            self.out.push(n as u8);
            self.out.extend_from_slice(&self.buffer[..n]);
            self.buffer[0] = 0;
            self.index = 0;
            self.bits_left = 8;
        }
    }
}

const HASH_FREE: i16 = -1;
const MAXSTR: usize = 1 << 12;
const HASHSIZE: i32 = 9973;
const HASHSTEP: i32 = 2039;

/// `LZWStringTable`.
struct StringTable {
    chr: Vec<i8>,
    nxt: Vec<i16>,
    hsh: Vec<i16>,
    num_strings: i16,
}

impl StringTable {
    fn hash(index: i16, b: i8) -> usize {
        (((((b as i32) << 8) as i16 as i32 ^ index as i32) & 0xFFFF) % HASHSIZE) as usize
    }

    fn add(&mut self, index: i16, b: i8) -> i32 {
        if self.num_strings as usize >= MAXSTR {
            return 0xFFFF;
        }
        let mut h = Self::hash(index, b);
        while self.hsh[h] != HASH_FREE {
            h = (h + HASHSTEP as usize) % HASHSIZE as usize;
        }
        let n = self.num_strings as usize;
        self.hsh[h] = self.num_strings;
        self.chr[n] = b;
        self.nxt[n] = index;
        self.num_strings += 1;
        n as i32
    }

    fn find(&self, index: i16, b: i8) -> i16 {
        if index == HASH_FREE {
            return (b as u8) as i16;
        }
        let mut h = Self::hash(index, b);
        loop {
            let n = self.hsh[h];
            if n == HASH_FREE {
                return HASH_FREE;
            }
            if self.nxt[n as usize] == index && self.chr[n as usize] == b {
                return n;
            }
            h = (h + HASHSTEP as usize) % HASHSIZE as usize;
        }
    }

    fn clear(&mut self, code_size: u32) {
        self.num_strings = 0;
        self.hsh.fill(HASH_FREE);
        for q in 0..(1 << code_size) + 2 {
            self.add(HASH_FREE, q as u8 as i8);
        }
    }
}

/// `LZWCompressor` for GIF.
struct Lzw {
    code_size: u32,
    clear_code: u32,
    num_bits: u32,
    limit: i32,
    prefix: i16,
    bf: BitFile,
    table: StringTable,
}

impl Lzw {
    fn new(code_size: u32) -> Lzw {
        let mut table = StringTable { chr: vec![0; MAXSTR], nxt: vec![0; MAXSTR], hsh: vec![0; HASHSIZE as usize], num_strings: 0 };
        table.clear(code_size);
        let mut lzw = Lzw {
            code_size,
            clear_code: 1 << code_size,
            num_bits: code_size + 1,
            limit: (1 << (code_size + 1)) - 1,
            prefix: HASH_FREE,
            bf: BitFile { out: Vec::new(), buffer: [0; 256], index: 0, bits_left: 8 },
            table,
        };
        lzw.bf.write_bits(lzw.clear_code, lzw.num_bits);
        lzw
    }

    fn compress(&mut self, data: &[u8]) {
        for &c in data {
            let c = c as i8;
            let index = self.table.find(self.prefix, c);
            if index != -1 {
                self.prefix = index;
            } else {
                self.bf.write_bits(self.prefix as u16 as u32, self.num_bits);
                if self.table.add(self.prefix, c) > self.limit {
                    if self.num_bits == 12 {
                        self.bf.write_bits(self.clear_code, self.num_bits);
                        self.table.clear(self.code_size);
                        self.num_bits = self.code_size + 1;
                    } else {
                        self.num_bits += 1;
                    }
                    self.limit = (1 << self.num_bits) - 1;
                }
                self.prefix = (c as u8) as i16;
            }
        }
    }

    fn flush(mut self) -> Vec<u8> {
        if self.prefix != -1 {
            self.bf.write_bits(self.prefix as u16 as u32, self.num_bits);
        }
        self.bf.write_bits(self.clear_code + 1, self.num_bits);
        self.bf.flush();
        self.bf.out
    }
}

/// `ImageIO.write(image, "gif", stream)`.
pub fn write(image: &BufferedImage) -> Vec<u8> {
    let (palette, indices) = PaletteBuilder::create_indexed_image(image);
    let size = palette.len().max(2).next_power_of_two();
    let bits_per_pixel = size.trailing_zeros() as u8;
    let (w, h) = (image.width(), image.height());
    let mut out = b"GIF89a".to_vec();
    out.extend_from_slice(&(w as u16).to_le_bytes());
    out.extend_from_slice(&(h as u16).to_le_bytes());
    out.extend_from_slice(&[0x80 | (7 << 4) | (bits_per_pixel - 1), 0, 0]);
    for i in 0..size {
        out.extend_from_slice(&palette[if i < palette.len() { i } else { 0 }]);
    }
    out.push(0x2c);
    for v in [0u16, 0, w as u16, h as u16] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.push(0x40 | (bits_per_pixel - 1));
    // `TYPE_BYTE_INDEXED` samples are 8 bits.
    out.push(8);
    let mut lzw = Lzw::new(8);
    for (start, step) in [(0, 8), (4, 8), (2, 4), (1, 2)] {
        for y in (start..h).step_by(step) {
            lzw.compress(&indices[y * w..(y + 1) * w]);
        }
    }
    out.extend_from_slice(&lzw.flush());
    out.push(0);
    out.push(0x3b);
    out
}
