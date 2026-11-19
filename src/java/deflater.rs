//! `java.util.zip.Deflater` and `CRC32` as the JDK runs them: the JDK bundles zlib 1.3.1 (its `libzip`), so a
//! `Deflater(level)` is zlib's `deflate()` with a zlib wrapper, a 32K window, `memLevel` 8 and the default
//! strategy. This is a port of zlib 1.3.1's `deflate.c` and `trees.c` for those settings, byte for byte (the PNG
//! encoder's `Deflater(4)` output is pinned by `tests/vectors/images.tsv`). Input arrives in one piece and is
//! finished at once: zlib's output does not depend on how `NO_FLUSH` input is split, since matching only runs with
//! `MIN_LOOKAHEAD` bytes ahead until the input ends.

const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
const MIN_LOOKAHEAD: usize = MAX_MATCH + MIN_MATCH + 1;
const W_BITS: usize = 15;
const W_SIZE: usize = 1 << W_BITS;
const W_MASK: usize = W_SIZE - 1;
const WINDOW_SIZE: usize = 2 * W_SIZE;
const MAX_DIST: usize = W_SIZE - MIN_LOOKAHEAD;
const HASH_BITS: usize = 8 + 7;
const HASH_SIZE: usize = 1 << HASH_BITS;
const HASH_MASK: usize = HASH_SIZE - 1;
const HASH_SHIFT: usize = HASH_BITS.div_ceil(MIN_MATCH);
const LIT_BUFSIZE: usize = 1 << (8 + 6);
const SYM_END: usize = LIT_BUFSIZE - 1;
const TOO_FAR: usize = 4096;
const WIN_INIT: usize = MAX_MATCH;
const NIL: usize = 0;

const LENGTH_CODES: usize = 29;
const LITERALS: usize = 256;
const L_CODES: usize = LITERALS + 1 + LENGTH_CODES;
const D_CODES: usize = 30;
const BL_CODES: usize = 19;
const HEAP_SIZE: usize = 2 * L_CODES + 1;
const MAX_BITS: usize = 15;
const MAX_BL_BITS: usize = 7;
const END_BLOCK: usize = 256;
const REP_3_6: usize = 16;
const REPZ_3_10: usize = 17;
const REPZ_11_138: usize = 18;

const EXTRA_LBITS: [u8; LENGTH_CODES] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const EXTRA_DBITS: [u8; D_CODES] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
const EXTRA_BLBITS: [u8; BL_CODES] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 7];
const BL_ORDER: [usize; BL_CODES] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

/// zlib's `configuration_table`: good, lazy, nice, chain, and whether the level uses `deflate_slow`.
const CONFIG: [(usize, usize, usize, usize, bool); 10] = [
    (0, 0, 0, 0, false),
    (4, 4, 8, 4, false),
    (4, 5, 16, 8, false),
    (4, 6, 32, 32, false),
    (4, 4, 16, 16, true),
    (8, 16, 32, 32, true),
    (8, 16, 128, 128, true),
    (8, 32, 128, 256, true),
    (32, 128, 258, 1024, true),
    (32, 258, 258, 4096, true),
];

/// `tr_static_init`'s tables.
struct Static {
    ltree: [Ct; L_CODES + 2],
    dtree: [Ct; D_CODES],
    dist_code: [u8; 512],
    length_code: [u8; MAX_MATCH - MIN_MATCH + 1],
    base_length: [u16; LENGTH_CODES],
    base_dist: [u16; D_CODES],
}

/// zlib's `ct_data`: `fc` is the frequency, later the code; `dl` the father node, later the bit length.
#[derive(Clone, Copy, Default)]
struct Ct {
    fc: u16,
    dl: u16,
}

fn bi_reverse(mut code: u32, mut len: u32) -> u32 {
    let mut res = 0;
    loop {
        res |= code & 1;
        code >>= 1;
        res <<= 1;
        len -= 1;
        if len == 0 {
            break;
        }
    }
    res >> 1
}

fn gen_codes(tree: &mut [Ct], max_code: isize, bl_count: &[u16; MAX_BITS + 1]) {
    let mut next_code = [0u16; MAX_BITS + 1];
    let mut code: u32 = 0;
    for bits in 1..=MAX_BITS {
        code = (code + bl_count[bits - 1] as u32) << 1;
        next_code[bits] = code as u16;
    }
    for t in tree.iter_mut().take((max_code + 1).max(0) as usize) {
        let len = t.dl as usize;
        if len == 0 {
            continue;
        }
        t.fc = bi_reverse(next_code[len] as u32, len as u32) as u16;
        next_code[len] = next_code[len].wrapping_add(1);
    }
}

fn statics() -> &'static Static {
    static TABLES: std::sync::OnceLock<Static> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut s = Static {
            ltree: [Ct::default(); L_CODES + 2],
            dtree: [Ct::default(); D_CODES],
            dist_code: [0; 512],
            length_code: [0; MAX_MATCH - MIN_MATCH + 1],
            base_length: [0; LENGTH_CODES],
            base_dist: [0; D_CODES],
        };
        let mut length = 0usize;
        let mut code = 0usize;
        while code < LENGTH_CODES - 1 {
            s.base_length[code] = length as u16;
            for _ in 0..(1 << EXTRA_LBITS[code]) {
                s.length_code[length] = code as u8;
                length += 1;
            }
            code += 1;
        }
        s.length_code[length - 1] = code as u8;
        let mut dist = 0usize;
        code = 0;
        while code < 16 {
            s.base_dist[code] = dist as u16;
            for _ in 0..(1 << EXTRA_DBITS[code]) {
                s.dist_code[dist] = code as u8;
                dist += 1;
            }
            code += 1;
        }
        dist >>= 7;
        while code < D_CODES {
            s.base_dist[code] = (dist << 7) as u16;
            for _ in 0..(1 << (EXTRA_DBITS[code] - 7)) {
                s.dist_code[256 + dist] = code as u8;
                dist += 1;
            }
            code += 1;
        }
        let mut bl_count = [0u16; MAX_BITS + 1];
        for n in 0..L_CODES + 2 {
            let len = match n {
                0..=143 => 8,
                144..=255 => 9,
                256..=279 => 7,
                _ => 8,
            };
            s.ltree[n].dl = len;
            bl_count[len as usize] += 1;
        }
        gen_codes(&mut s.ltree, (L_CODES + 1) as isize, &bl_count);
        for n in 0..D_CODES {
            s.dtree[n].dl = 5;
            s.dtree[n].fc = bi_reverse(n as u32, 5) as u16;
        }
        s
    })
}

fn d_code(st: &Static, dist: usize) -> usize {
    if dist < 256 { st.dist_code[dist] as usize } else { st.dist_code[256 + (dist >> 7)] as usize }
}

#[derive(Clone, Copy, PartialEq)]
enum TreeKind {
    Lit,
    Dist,
    Bl,
}

struct State<'a> {
    input: &'a [u8],
    next_in: usize,
    level: usize,
    out: Vec<u8>,
    window: Vec<u8>,
    prev: Vec<u16>,
    head: Vec<u16>,
    ins_h: usize,
    block_start: isize,
    match_length: usize,
    prev_match: usize,
    match_available: bool,
    strstart: usize,
    match_start: usize,
    lookahead: usize,
    prev_length: usize,
    max_chain_length: usize,
    max_lazy_match: usize,
    good_match: usize,
    nice_match: usize,
    insert: usize,
    high_water: usize,
    dyn_ltree: Vec<Ct>,
    dyn_dtree: Vec<Ct>,
    bl_tree: Vec<Ct>,
    l_max_code: isize,
    d_max_code: isize,
    bl_max_code: isize,
    bl_count: [u16; MAX_BITS + 1],
    heap: [usize; HEAP_SIZE],
    heap_len: usize,
    heap_max: usize,
    depth: [u8; HEAP_SIZE],
    sym_buf: Vec<(u16, u8)>,
    opt_len: u64,
    static_len: u64,
    bi_buf: u16,
    bi_valid: u32,
    adler: u32,
}

impl State<'_> {
    fn tree(&mut self, kind: TreeKind) -> &mut Vec<Ct> {
        match kind {
            TreeKind::Lit => &mut self.dyn_ltree,
            TreeKind::Dist => &mut self.dyn_dtree,
            TreeKind::Bl => &mut self.bl_tree,
        }
    }

    fn update_hash(&mut self, c: u8) {
        self.ins_h = ((self.ins_h << HASH_SHIFT) ^ c as usize) & HASH_MASK;
    }

    fn insert_string(&mut self, s: usize) -> usize {
        self.update_hash(self.window[s + MIN_MATCH - 1]);
        let head = self.head[self.ins_h] as usize;
        self.prev[s & W_MASK] = head as u16;
        self.head[self.ins_h] = s as u16;
        head
    }

    fn slide_hash(&mut self) {
        for p in self.head.iter_mut().chain(self.prev.iter_mut()) {
            let m = *p as usize;
            *p = if m >= W_SIZE { (m - W_SIZE) as u16 } else { NIL as u16 };
        }
    }

    fn read_buf(&mut self, at: usize, size: usize) -> usize {
        let len = (self.input.len() - self.next_in).min(size);
        if len == 0 {
            return 0;
        }
        let chunk = &self.input[self.next_in..self.next_in + len];
        self.window[at..at + len].copy_from_slice(chunk);
        self.adler = adler32(self.adler, chunk);
        self.next_in += len;
        len
    }

    fn avail_in(&self) -> usize {
        self.input.len() - self.next_in
    }

    fn fill_window(&mut self) {
        loop {
            let mut more = WINDOW_SIZE - self.lookahead - self.strstart;
            if self.strstart >= W_SIZE + MAX_DIST {
                self.window.copy_within(W_SIZE..W_SIZE + W_SIZE - more, 0);
                self.match_start = self.match_start.wrapping_sub(W_SIZE);
                self.strstart -= W_SIZE;
                self.block_start -= W_SIZE as isize;
                if self.insert > self.strstart {
                    self.insert = self.strstart;
                }
                self.slide_hash();
                more += W_SIZE;
            }
            if self.avail_in() == 0 {
                break;
            }
            let n = self.read_buf(self.strstart + self.lookahead, more);
            self.lookahead += n;
            if self.lookahead + self.insert >= MIN_MATCH {
                let mut s = self.strstart - self.insert;
                self.ins_h = self.window[s] as usize;
                self.update_hash(self.window[s + 1]);
                while self.insert > 0 {
                    self.update_hash(self.window[s + MIN_MATCH - 1]);
                    self.prev[s & W_MASK] = self.head[self.ins_h];
                    self.head[self.ins_h] = s as u16;
                    s += 1;
                    self.insert -= 1;
                    if self.lookahead + self.insert < MIN_MATCH {
                        break;
                    }
                }
            }
            if !(self.lookahead < MIN_LOOKAHEAD && self.avail_in() != 0) {
                break;
            }
        }
        if self.high_water < WINDOW_SIZE {
            let curr = self.strstart + self.lookahead;
            if self.high_water < curr {
                let init = (WINDOW_SIZE - curr).min(WIN_INIT);
                self.window[curr..curr + init].fill(0);
                self.high_water = curr + init;
            } else if self.high_water < curr + WIN_INIT {
                let init = (curr + WIN_INIT - self.high_water).min(WINDOW_SIZE - self.high_water);
                self.window[self.high_water..self.high_water + init].fill(0);
                self.high_water += init;
            }
        }
    }

    fn longest_match(&mut self, mut cur_match: usize) -> usize {
        let mut chain_length = self.max_chain_length;
        let scan = self.strstart;
        let mut best_len = self.prev_length;
        let mut nice_match = self.nice_match;
        let limit = if self.strstart > MAX_DIST { self.strstart - MAX_DIST } else { NIL };
        let w = &self.window;
        let mut scan_end1 = w[scan + best_len - 1];
        let mut scan_end = w[scan + best_len];
        if self.prev_length >= self.good_match {
            chain_length >>= 2;
        }
        if nice_match > self.lookahead {
            nice_match = self.lookahead;
        }
        loop {
            let m = cur_match;
            let candidate = w[m + best_len] == scan_end && w[m + best_len - 1] == scan_end1 && w[m] == w[scan] && w[m + 1] == w[scan + 1];
            if candidate {
                // scan[2] == match[2] since the hash keys are equal; compare from 3 on, up to MAX_MATCH.
                let mut len = 3;
                while len < MAX_MATCH && w[scan + len] == w[m + len] {
                    len += 1;
                }
                if len > best_len {
                    self.match_start = cur_match;
                    best_len = len;
                    if len >= nice_match {
                        break;
                    }
                    scan_end1 = w[scan + best_len - 1];
                    scan_end = w[scan + best_len];
                }
            }
            cur_match = self.prev[cur_match & W_MASK] as usize;
            if cur_match <= limit {
                break;
            }
            chain_length -= 1;
            if chain_length == 0 {
                break;
            }
        }
        best_len.min(self.lookahead)
    }

    fn tally_lit(&mut self, c: u8) -> bool {
        self.sym_buf.push((0, c));
        self.dyn_ltree[c as usize].fc += 1;
        self.sym_buf.len() == SYM_END
    }

    fn tally_dist(&mut self, dist: usize, len: usize) -> bool {
        let st = statics();
        self.sym_buf.push((dist as u16, len as u8));
        let d = dist - 1;
        self.dyn_ltree[st.length_code[len] as usize + LITERALS + 1].fc += 1;
        self.dyn_dtree[d_code(st, d)].fc += 1;
        self.sym_buf.len() == SYM_END
    }

    fn flush_block(&mut self, last: bool) {
        let stored = if self.block_start >= 0 { Some(self.block_start as usize) } else { None };
        let stored_len = (self.strstart as isize - self.block_start) as usize;
        self.tr_flush_block(stored, stored_len, last);
        self.block_start = self.strstart as isize;
    }

    fn deflate_fast(&mut self) {
        loop {
            if self.lookahead < MIN_LOOKAHEAD {
                self.fill_window();
                if self.lookahead == 0 {
                    break;
                }
            }
            let mut hash_head = NIL;
            if self.lookahead >= MIN_MATCH {
                hash_head = self.insert_string(self.strstart);
            }
            if hash_head != NIL && self.strstart - hash_head <= MAX_DIST {
                self.match_length = self.longest_match(hash_head);
            }
            let bflush;
            if self.match_length >= MIN_MATCH {
                bflush = self.tally_dist(self.strstart - self.match_start, self.match_length - MIN_MATCH);
                self.lookahead -= self.match_length;
                if self.match_length <= self.max_lazy_match && self.lookahead >= MIN_MATCH {
                    self.match_length -= 1;
                    loop {
                        self.strstart += 1;
                        self.insert_string(self.strstart);
                        self.match_length -= 1;
                        if self.match_length == 0 {
                            break;
                        }
                    }
                    self.strstart += 1;
                } else {
                    self.strstart += self.match_length;
                    self.match_length = 0;
                    self.ins_h = self.window[self.strstart] as usize;
                    self.update_hash(self.window[self.strstart + 1]);
                }
            } else {
                bflush = self.tally_lit(self.window[self.strstart]);
                self.lookahead -= 1;
                self.strstart += 1;
            }
            if bflush {
                self.flush_block(false);
            }
        }
        self.insert = self.strstart.min(MIN_MATCH - 1);
        self.flush_block(true);
    }

    fn deflate_slow(&mut self) {
        loop {
            if self.lookahead < MIN_LOOKAHEAD {
                self.fill_window();
                if self.lookahead == 0 {
                    break;
                }
            }
            let mut hash_head = NIL;
            if self.lookahead >= MIN_MATCH {
                hash_head = self.insert_string(self.strstart);
            }
            self.prev_length = self.match_length;
            self.prev_match = self.match_start;
            self.match_length = MIN_MATCH - 1;
            if hash_head != NIL && self.prev_length < self.max_lazy_match && self.strstart - hash_head <= MAX_DIST {
                self.match_length = self.longest_match(hash_head);
                if self.match_length <= 5 && self.match_length == MIN_MATCH && self.strstart - self.match_start > TOO_FAR {
                    self.match_length = MIN_MATCH - 1;
                }
            }
            if self.prev_length >= MIN_MATCH && self.match_length <= self.prev_length {
                let max_insert = self.strstart + self.lookahead - MIN_MATCH;
                let bflush = self.tally_dist(self.strstart - 1 - self.prev_match, self.prev_length - MIN_MATCH);
                self.lookahead -= self.prev_length - 1;
                self.prev_length -= 2;
                loop {
                    self.strstart += 1;
                    if self.strstart <= max_insert {
                        self.insert_string(self.strstart);
                    }
                    self.prev_length -= 1;
                    if self.prev_length == 0 {
                        break;
                    }
                }
                self.match_available = false;
                self.match_length = MIN_MATCH - 1;
                self.strstart += 1;
                if bflush {
                    self.flush_block(false);
                }
            } else if self.match_available {
                if self.tally_lit(self.window[self.strstart - 1]) {
                    self.flush_block(false);
                }
                self.strstart += 1;
                self.lookahead -= 1;
            } else {
                self.match_available = true;
                self.strstart += 1;
                self.lookahead -= 1;
            }
        }
        if self.match_available {
            self.tally_lit(self.window[self.strstart - 1]);
            self.match_available = false;
        }
        self.insert = self.strstart.min(MIN_MATCH - 1);
        self.flush_block(true);
    }

    // trees.c

    fn send_bits(&mut self, value: u32, length: u32) {
        if self.bi_valid > 16 - length {
            self.bi_buf |= (value << self.bi_valid) as u16;
            let b = self.bi_buf;
            self.out.extend_from_slice(&b.to_le_bytes());
            self.bi_buf = (value >> (16 - self.bi_valid)) as u16;
            self.bi_valid = self.bi_valid + length - 16;
        } else {
            self.bi_buf |= (value << self.bi_valid) as u16;
            self.bi_valid += length;
        }
    }

    fn send_code(&mut self, c: usize, tree: &[Ct]) {
        self.send_bits(tree[c].fc as u32, tree[c].dl as u32);
    }

    fn bi_windup(&mut self) {
        if self.bi_valid > 8 {
            let b = self.bi_buf;
            self.out.extend_from_slice(&b.to_le_bytes());
        } else if self.bi_valid > 0 {
            self.out.push(self.bi_buf as u8);
        }
        self.bi_buf = 0;
        self.bi_valid = 0;
    }

    fn init_block(&mut self) {
        for t in self.dyn_ltree.iter_mut().take(L_CODES) {
            t.fc = 0;
        }
        for t in self.dyn_dtree.iter_mut().take(D_CODES) {
            t.fc = 0;
        }
        for t in self.bl_tree.iter_mut().take(BL_CODES) {
            t.fc = 0;
        }
        self.dyn_ltree[END_BLOCK].fc = 1;
        self.opt_len = 0;
        self.static_len = 0;
        self.sym_buf.clear();
    }

    fn smaller(tree: &[Ct], depth: &[u8], n: usize, m: usize) -> bool {
        tree[n].fc < tree[m].fc || (tree[n].fc == tree[m].fc && depth[n] <= depth[m])
    }

    fn pqdownheap(&mut self, kind: TreeKind, mut k: usize) {
        let tree = std::mem::take(self.tree(kind));
        let v = self.heap[k];
        let mut j = k << 1;
        while j <= self.heap_len {
            if j < self.heap_len && Self::smaller(&tree, &self.depth, self.heap[j + 1], self.heap[j]) {
                j += 1;
            }
            if Self::smaller(&tree, &self.depth, v, self.heap[j]) {
                break;
            }
            self.heap[k] = self.heap[j];
            k = j;
            j <<= 1;
        }
        self.heap[k] = v;
        *self.tree(kind) = tree;
    }

    fn descriptor(kind: TreeKind) -> (Option<&'static [Ct]>, &'static [u8], usize, usize, usize) {
        let st = statics();
        match kind {
            TreeKind::Lit => (Some(&st.ltree[..]), &EXTRA_LBITS[..], LITERALS + 1, L_CODES, MAX_BITS),
            TreeKind::Dist => (Some(&st.dtree[..]), &EXTRA_DBITS[..], 0, D_CODES, MAX_BITS),
            TreeKind::Bl => (None, &EXTRA_BLBITS[..], 0, BL_CODES, MAX_BL_BITS),
        }
    }

    fn max_code(&mut self, kind: TreeKind) -> &mut isize {
        match kind {
            TreeKind::Lit => &mut self.l_max_code,
            TreeKind::Dist => &mut self.d_max_code,
            TreeKind::Bl => &mut self.bl_max_code,
        }
    }

    fn gen_bitlen(&mut self, kind: TreeKind) {
        let (stree, extra, base, _, max_length) = Self::descriptor(kind);
        let max_code = *self.max_code(kind);
        let mut tree = std::mem::take(self.tree(kind));
        let mut overflow = 0i32;
        self.bl_count = [0; MAX_BITS + 1];
        tree[self.heap[self.heap_max]].dl = 0;
        let mut h = self.heap_max + 1;
        while h < HEAP_SIZE {
            let n = self.heap[h];
            let mut bits = tree[tree[n].dl as usize].dl as usize + 1;
            if bits > max_length {
                bits = max_length;
                overflow += 1;
            }
            tree[n].dl = bits as u16;
            h += 1;
            if n as isize > max_code {
                continue;
            }
            self.bl_count[bits] += 1;
            let xbits = if n >= base { extra[n - base] as u64 } else { 0 };
            let f = tree[n].fc as u64;
            self.opt_len += f * (bits as u64 + xbits);
            if let Some(stree) = stree {
                self.static_len += f * (stree[n].dl as u64 + xbits);
            }
        }
        if overflow != 0 {
            loop {
                let mut bits = max_length - 1;
                while self.bl_count[bits] == 0 {
                    bits -= 1;
                }
                self.bl_count[bits] -= 1;
                self.bl_count[bits + 1] += 2;
                self.bl_count[max_length] -= 1;
                overflow -= 2;
                if overflow <= 0 {
                    break;
                }
            }
            let mut h = HEAP_SIZE;
            for bits in (1..=max_length).rev() {
                let mut n = self.bl_count[bits];
                while n != 0 {
                    h -= 1;
                    let m = self.heap[h];
                    if m as isize > max_code {
                        continue;
                    }
                    if tree[m].dl as usize != bits {
                        self.opt_len = self.opt_len.wrapping_add((bits as u64).wrapping_sub(tree[m].dl as u64).wrapping_mul(tree[m].fc as u64));
                        tree[m].dl = bits as u16;
                    }
                    n -= 1;
                }
            }
        }
        *self.tree(kind) = tree;
    }

    fn build_tree(&mut self, kind: TreeKind) {
        let (stree, _, _, elems, _) = Self::descriptor(kind);
        let mut max_code: isize = -1;
        self.heap_len = 0;
        self.heap_max = HEAP_SIZE;
        {
            let mut tree = std::mem::take(self.tree(kind));
            for (n, t) in tree.iter_mut().enumerate().take(elems) {
                if t.fc != 0 {
                    self.heap_len += 1;
                    self.heap[self.heap_len] = n;
                    max_code = n as isize;
                    self.depth[n] = 0;
                } else {
                    t.dl = 0;
                }
            }
            while self.heap_len < 2 {
                let node = if max_code < 2 {
                    max_code += 1;
                    max_code as usize
                } else {
                    0
                };
                self.heap_len += 1;
                self.heap[self.heap_len] = node;
                tree[node].fc = 1;
                self.depth[node] = 0;
                self.opt_len = self.opt_len.wrapping_sub(1);
                if let Some(stree) = stree {
                    self.static_len = self.static_len.wrapping_sub(stree[node].dl as u64);
                }
            }
            *self.tree(kind) = tree;
        }
        *self.max_code(kind) = max_code;
        let mut n = self.heap_len / 2;
        while n >= 1 {
            self.pqdownheap(kind, n);
            n -= 1;
        }
        let mut node = elems;
        loop {
            // pqremove
            let n = self.heap[1];
            self.heap[1] = self.heap[self.heap_len];
            self.heap_len -= 1;
            self.pqdownheap(kind, 1);
            let m = self.heap[1];
            self.heap_max -= 1;
            self.heap[self.heap_max] = n;
            self.heap_max -= 1;
            self.heap[self.heap_max] = m;
            let depth = self.depth[n].max(self.depth[m]) + 1;
            let tree = self.tree(kind);
            tree[node].fc = tree[n].fc + tree[m].fc;
            tree[n].dl = node as u16;
            tree[m].dl = node as u16;
            self.depth[node] = depth;
            self.heap[1] = node;
            node += 1;
            self.pqdownheap(kind, 1);
            if self.heap_len < 2 {
                break;
            }
        }
        self.heap_max -= 1;
        self.heap[self.heap_max] = self.heap[1];
        self.gen_bitlen(kind);
        let bl_count = self.bl_count;
        gen_codes(self.tree(kind), max_code, &bl_count);
    }

    fn scan_tree(&mut self, kind: TreeKind) {
        let max_code = *self.max_code(kind);
        let tree = self.tree(kind);
        let mut prevlen: i32 = -1;
        let mut nextlen = tree[0].dl as i32;
        let mut count = 0;
        let (mut max_count, mut min_count) = (7, 4);
        if nextlen == 0 {
            max_count = 138;
            min_count = 3;
        }
        tree[(max_code + 1) as usize].dl = 0xffff;
        let lens: Vec<i32> = tree.iter().map(|t| t.dl as i32).collect();
        for n in 0..=max_code as usize {
            let curlen = nextlen;
            nextlen = lens[n + 1];
            count += 1;
            if count < max_count && curlen == nextlen {
                continue;
            } else if count < min_count {
                self.bl_tree[curlen as usize].fc += count as u16;
            } else if curlen != 0 {
                if curlen != prevlen {
                    self.bl_tree[curlen as usize].fc += 1;
                }
                self.bl_tree[REP_3_6].fc += 1;
            } else if count <= 10 {
                self.bl_tree[REPZ_3_10].fc += 1;
            } else {
                self.bl_tree[REPZ_11_138].fc += 1;
            }
            count = 0;
            prevlen = curlen;
            if nextlen == 0 {
                max_count = 138;
                min_count = 3;
            } else if curlen == nextlen {
                max_count = 6;
                min_count = 3;
            } else {
                max_count = 7;
                min_count = 4;
            }
        }
    }

    fn send_tree(&mut self, kind: TreeKind, max_code: usize) {
        let lens: Vec<i32> = self.tree(kind).iter().map(|t| t.dl as i32).collect();
        let bl = self.bl_tree.clone();
        let mut prevlen: i32 = -1;
        let mut nextlen = lens[0];
        let mut count = 0;
        let (mut max_count, mut min_count) = (7, 4);
        if nextlen == 0 {
            max_count = 138;
            min_count = 3;
        }
        for n in 0..=max_code {
            let curlen = nextlen;
            nextlen = lens[n + 1];
            count += 1;
            if count < max_count && curlen == nextlen {
                continue;
            } else if count < min_count {
                while count != 0 {
                    self.send_code(curlen as usize, &bl);
                    count -= 1;
                }
            } else if curlen != 0 {
                if curlen != prevlen {
                    self.send_code(curlen as usize, &bl);
                    count -= 1;
                }
                self.send_code(REP_3_6, &bl);
                self.send_bits((count - 3) as u32, 2);
            } else if count <= 10 {
                self.send_code(REPZ_3_10, &bl);
                self.send_bits((count - 3) as u32, 3);
            } else {
                self.send_code(REPZ_11_138, &bl);
                self.send_bits((count - 11) as u32, 7);
            }
            count = 0;
            prevlen = curlen;
            if nextlen == 0 {
                max_count = 138;
                min_count = 3;
            } else if curlen == nextlen {
                max_count = 6;
                min_count = 3;
            } else {
                max_count = 7;
                min_count = 4;
            }
        }
    }

    fn build_bl_tree(&mut self) -> usize {
        self.scan_tree(TreeKind::Lit);
        self.scan_tree(TreeKind::Dist);
        self.build_tree(TreeKind::Bl);
        let mut max_blindex = BL_CODES - 1;
        while max_blindex >= 3 {
            if self.bl_tree[BL_ORDER[max_blindex]].dl != 0 {
                break;
            }
            max_blindex -= 1;
        }
        self.opt_len = self.opt_len.wrapping_add(3 * (max_blindex as u64 + 1) + 5 + 5 + 4);
        max_blindex
    }

    fn send_all_trees(&mut self, lcodes: usize, dcodes: usize, blcodes: usize) {
        self.send_bits((lcodes - 257) as u32, 5);
        self.send_bits((dcodes - 1) as u32, 5);
        self.send_bits((blcodes - 4) as u32, 4);
        for rank in BL_ORDER.iter().take(blcodes) {
            let len = self.bl_tree[*rank].dl as u32;
            self.send_bits(len, 3);
        }
        self.send_tree(TreeKind::Lit, lcodes - 1);
        self.send_tree(TreeKind::Dist, dcodes - 1);
    }

    fn compress_block(&mut self, ltree: &[Ct], dtree: &[Ct]) {
        let st = statics();
        let syms = std::mem::take(&mut self.sym_buf);
        for &(dist, lc) in &syms {
            let mut lc = lc as usize;
            let mut dist = dist as usize;
            if dist == 0 {
                self.send_code(lc, ltree);
            } else {
                let code = st.length_code[lc] as usize;
                self.send_code(code + LITERALS + 1, ltree);
                let extra = EXTRA_LBITS[code] as u32;
                if extra != 0 {
                    lc -= st.base_length[code] as usize;
                    self.send_bits(lc as u32, extra);
                }
                dist -= 1;
                let code = d_code(st, dist);
                self.send_code(code, dtree);
                let extra = EXTRA_DBITS[code] as u32;
                if extra != 0 {
                    dist -= st.base_dist[code] as usize;
                    self.send_bits(dist as u32, extra);
                }
            }
        }
        self.sym_buf = syms;
        self.send_code(END_BLOCK, ltree);
    }

    fn stored_block(&mut self, buf: usize, stored_len: usize, last: bool) {
        self.send_bits(last as u32, 3);
        self.bi_windup();
        self.out.extend_from_slice(&(stored_len as u16).to_le_bytes());
        self.out.extend_from_slice(&(!(stored_len as u16)).to_le_bytes());
        let data = self.window[buf..buf + stored_len].to_vec();
        self.out.extend_from_slice(&data);
    }

    fn tr_flush_block(&mut self, buf: Option<usize>, stored_len: usize, last: bool) {
        let st = statics();
        let mut max_blindex = 0;
        let (mut opt_lenb, static_lenb);
        if self.level > 0 {
            self.build_tree(TreeKind::Lit);
            self.build_tree(TreeKind::Dist);
            max_blindex = self.build_bl_tree();
            opt_lenb = (self.opt_len.wrapping_add(3 + 7)) >> 3;
            static_lenb = (self.static_len.wrapping_add(3 + 7)) >> 3;
            if static_lenb <= opt_lenb {
                opt_lenb = static_lenb;
            }
        } else {
            opt_lenb = stored_len as u64 + 5;
            static_lenb = opt_lenb;
        }
        if stored_len as u64 + 4 <= opt_lenb && buf.is_some() {
            self.stored_block(buf.unwrap_or(0), stored_len, last);
        } else if static_lenb == opt_lenb {
            self.send_bits((1 << 1) + last as u32, 3);
            self.compress_block(&st.ltree, &st.dtree);
        } else {
            self.send_bits((2 << 1) + last as u32, 3);
            let (l, d) = (self.l_max_code as usize + 1, self.d_max_code as usize + 1);
            self.send_all_trees(l, d, max_blindex + 1);
            let ltree = self.dyn_ltree.clone();
            let dtree = self.dyn_dtree.clone();
            self.compress_block(&ltree, &dtree);
        }
        self.init_block();
        if last {
            self.bi_windup();
        }
    }
}

/// `Adler32`'s update.
pub fn adler32(adler: u32, data: &[u8]) -> u32 {
    let (mut a, mut b) = (adler & 0xffff, adler >> 16);
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// `new Deflater(level)` given all of `input`, then `finish()`, drained: the zlib stream it writes. Levels 1 to 9
/// (level 0, `deflate_stored`, is not reached by Datafaker).
pub fn deflate(level: usize, input: &[u8]) -> Vec<u8> {
    assert!((1..=9).contains(&level), "Deflater level {level} is not ported");
    let (good, lazy, nice, chain, slow) = CONFIG[level];
    let mut s = State {
        input,
        next_in: 0,
        level,
        out: Vec::with_capacity(input.len() / 2 + 64),
        window: vec![0; WINDOW_SIZE],
        prev: vec![0; W_SIZE],
        head: vec![0; HASH_SIZE],
        ins_h: 0,
        block_start: 0,
        match_length: MIN_MATCH - 1,
        prev_match: 0,
        match_available: false,
        strstart: 0,
        match_start: 0,
        lookahead: 0,
        prev_length: MIN_MATCH - 1,
        max_chain_length: chain,
        max_lazy_match: lazy,
        good_match: good,
        nice_match: nice,
        insert: 0,
        high_water: 0,
        dyn_ltree: vec![Ct::default(); HEAP_SIZE],
        dyn_dtree: vec![Ct::default(); 2 * D_CODES + 1],
        bl_tree: vec![Ct::default(); 2 * BL_CODES + 1],
        l_max_code: 0,
        d_max_code: 0,
        bl_max_code: 0,
        bl_count: [0; MAX_BITS + 1],
        heap: [0; HEAP_SIZE],
        heap_len: 0,
        heap_max: 0,
        depth: [0; HEAP_SIZE],
        sym_buf: Vec::with_capacity(SYM_END),
        opt_len: 0,
        static_len: 0,
        bi_buf: 0,
        bi_valid: 0,
        adler: 1,
    };
    s.init_block();
    let level_flags = if level < 2 {
        0
    } else if level < 6 {
        1
    } else if level == 6 {
        2
    } else {
        3
    };
    let mut header: u32 = ((8 + ((W_BITS as u32 - 8) << 4)) << 8) | (level_flags << 6);
    header += 31 - (header % 31);
    s.out.extend_from_slice(&(header as u16).to_be_bytes());
    if slow { s.deflate_slow() } else { s.deflate_fast() }
    let adler = s.adler;
    s.out.extend_from_slice(&adler.to_be_bytes());
    s.out
}

/// `java.util.zip.CRC32`.
#[derive(Debug, Clone, Default)]
pub struct Crc32 {
    crc: u32,
}

impl Crc32 {
    /// `new CRC32()`.
    pub fn new() -> Crc32 {
        Crc32 { crc: 0 }
    }

    /// `CRC32.update(byte[])`.
    pub fn update(&mut self, data: &[u8]) {
        static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
        let table = TABLE.get_or_init(|| {
            let mut t = [0u32; 256];
            for (n, e) in t.iter_mut().enumerate() {
                let mut c = n as u32;
                for _ in 0..8 {
                    c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
                }
                *e = c;
            }
            t
        });
        let mut c = !self.crc;
        for &b in data {
            c = table[((c ^ b as u32) & 0xff) as usize] ^ (c >> 8);
        }
        self.crc = !c;
    }

    /// `CRC32.getValue()`.
    pub fn value(&self) -> u32 {
        self.crc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_answer() {
        let mut c = Crc32::new();
        c.update(b"123456789");
        assert_eq!(c.value(), 0xcbf4_3926);
    }

    #[test]
    fn adler32_known_answer() {
        assert_eq!(adler32(1, b"Wikipedia"), 0x11e6_0398);
    }
}
