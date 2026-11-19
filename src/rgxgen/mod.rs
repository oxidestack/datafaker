//! The generation path of RgxGen 3.1 (`com.github.curiousoddman:rgxgen`), which Datafaker 2.7.0 shades and calls as
//! `RgxGen.parse(regex).generate(random)` for `regexify`: `DefaultTreeBuilder` (with `CharIterator`,
//! `ConstantsProvider` and `DefaultNodeCreator`), the nodes, `SymbolSet` with `Util`'s range compaction and
//! inversion, `SymbolSetIndexer`, and `GenerationVisitor` with default properties (`INFINITE_PATTERN_REPETITION`
//! 100, case-sensitive, `\s` = space and tab, `.` = printable ASCII). Every draw is `Random.nextInt(bound)`, in
//! RgxGen's order, so output is identical for the same seed. Parse errors carry RgxGen's (and the JDK's) exception
//! class names and messages.
//!
//! Supported: literals and escapes (`\x..`, `\x{..}`, `\u....`, `\Q..\E`, `\b`/`\B` ignored), `.`, `\d \D \s \S
//! \w \W`, `\p{..}`/`\P{..}` with RgxGen's Unicode category tables (`unicode.rs`, generated from its
//! `UnicodeCategory` enum), character classes with ranges, negation and nested shorthand classes, groups
//! (capturing, named, non-capturing, lookahead/lookbehind, which RgxGen generates as plain groups), back
//! references, alternation, `* + ? {n} {n,} {n,m}` (a lazy or possessive suffix is a second quantifier to RgxGen),
//! `^`/`$` with RgxGen's placement checks.
//!
//! Not supported: a negative lookahead/lookbehind (`(?!..)`, `(?<!..)`) whose content contains an alternation.
//! RgxGen's `NotMatchingGenerationVisitor` re-checks alternatives with `java.util.regex`, which is not ported; such
//! patterns fail to parse here with `UnsupportedOperationException`. Other negative groups are generated as RgxGen
//! does. `RgxGenProperties` (options), `iterateUnique` and `generateNotMatching` are not ported.

mod unicode;

use std::collections::HashMap;

use crate::java::RandomGenerator;
use crate::{Error, Result};

type Str = Vec<u16>;

/// `SymbolRange`: `from..=to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolRange {
    from: i32,
    to: i32,
}

impl SymbolRange {
    fn size(self) -> i32 {
        self.to - self.from + 1
    }
}

fn range(from: i32, to: i32) -> SymbolRange {
    SymbolRange { from, to }
}

const SPACE: i32 = 32;
const DEL: i32 = 127;
const ASCII_SYMBOL_RANGE: SymbolRange = SymbolRange { from: SPACE, to: DEL - 1 };
const UNICODE_SYMBOL_RANGE: SymbolRange = SymbolRange { from: SPACE, to: 0xD800 - 1 };
const INFINITE_PATTERN_REPETITION: i32 = 100;

/// `RgxGenCharsDefinition`.
#[derive(Debug, Clone, Default)]
struct CharsDefinition {
    ranges: Vec<SymbolRange>,
    chars: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchType {
    Positive,
    Negative,
}

/// `SymbolSet` with its `SymbolSetIndexer`.
#[derive(Debug, Clone)]
struct SymbolSet {
    is_ascii: bool,
    pattern: Str,
    ranges: Vec<SymbolRange>,
    symbols: Vec<u16>,
    negative_exclusion: Option<CharsDefinition>,
    range_offsets: Vec<i32>,
    size: i32,
}

impl SymbolSet {
    fn new(
        pattern: Str,
        positive: CharsDefinition,
        negative: Option<CharsDefinition>,
        match_type: MatchType,
        universe: SymbolRange,
    ) -> SymbolSet {
        let (ranges, symbols) = if match_type == MatchType::Positive {
            compact(&positive.ranges, &positive.chars)
        } else {
            let defs = negative.as_ref().unwrap_or(&positive);
            let (r, s) = compact(&defs.ranges, &defs.chars);
            invert(&r, &s, universe)
        };
        let mut size = symbols.len() as i32;
        let mut range_offsets = Vec::new();
        if !ranges.is_empty() {
            let mut current = 0;
            range_offsets.push(0);
            for r in &ranges[..ranges.len() - 1] {
                size += r.size();
                current += r.size();
                range_offsets.push(current);
            }
            size += ranges[ranges.len() - 1].size();
        }
        SymbolSet {
            is_ascii: universe == ASCII_SYMBOL_RANGE,
            pattern,
            ranges,
            symbols,
            negative_exclusion: negative,
            range_offsets,
            size,
        }
    }

    fn ascii(pattern: &str, ranges: Vec<SymbolRange>, chars: Vec<u16>, match_type: MatchType) -> SymbolSet {
        SymbolSet::new(s(pattern), CharsDefinition { ranges, chars }, None, match_type, ASCII_SYMBOL_RANGE)
    }

    /// `SymbolSet.getInvertedNode()`.
    fn inverted(&self) -> SymbolSet {
        let mut pattern: Str = s("[^");
        pattern.extend_from_slice(self.pattern.get(1..).unwrap_or(&[]));
        let universe = if self.is_ascii { ASCII_SYMBOL_RANGE } else { UNICODE_SYMBOL_RANGE };
        let defs = CharsDefinition { ranges: self.ranges.clone(), chars: self.symbols.clone() };
        SymbolSet::new(pattern, defs, None, MatchType::Negative, universe)
    }

    /// `SymbolSetIndexer.get(seed)`.
    fn get(&self, mut seed: i32) -> u16 {
        if (seed as usize) < self.symbols.len() {
            return self.symbols[seed as usize];
        }
        seed -= self.symbols.len() as i32;
        if seed == 0 {
            return self.ranges[0].from as u16;
        }
        let i = match self.range_offsets.binary_search(&seed) {
            Ok(i) => i,
            Err(insertion) => insertion - 1,
        };
        (self.ranges[i].from + seed - self.range_offsets[i]) as u16
    }
}

/// `Util.compactOverlappingRangesAndSymbols`.
fn compact(ranges: &[SymbolRange], symbols: &[u16]) -> (Vec<SymbolRange>, Vec<u16>) {
    let mut sorted: Vec<SymbolRange> =
        ranges.iter().copied().chain(symbols.iter().map(|c| range(*c as i32, *c as i32))).collect();
    sorted.sort_by_key(|r| r.from);
    if sorted.len() == 1 {
        return (ranges.to_vec(), symbols.to_vec());
    }
    let within = |l: SymbolRange, r: SymbolRange| l.from <= r.from && l.to >= r.to;
    let continues = |l: SymbolRange, r: SymbolRange| l.from <= r.from && r.from <= l.to + 1 && l.to < r.to;
    let mut i = 1;
    while i < sorted.len() {
        let (a, b) = (sorted[i - 1], sorted[i]);
        if within(a, b) {
            sorted.remove(i);
        } else if within(b, a) {
            sorted.remove(i - 1);
        } else if continues(a, b) {
            sorted.remove(i);
            sorted[i - 1] = range(a.from, b.to);
        } else {
            i += 1;
        }
    }
    let mut out_ranges = Vec::new();
    let mut out_symbols = Vec::new();
    for r in sorted {
        if r.from == r.to {
            out_symbols.push(r.from as u16);
        } else {
            out_ranges.push(r);
        }
    }
    (out_ranges, out_symbols)
}

/// `Util.invertSymbolsAndRanges`.
fn invert(ranges: &[SymbolRange], symbols: &[u16], all: SymbolRange) -> (Vec<SymbolRange>, Vec<u16>) {
    let mut sorted: Vec<SymbolRange> = ranges.iter().copied().filter(|r| r.to >= all.from && r.from <= all.to).collect();
    sorted.extend(symbols.iter().map(|c| *c as i32).filter(|c| all.from <= *c && *c <= all.to).map(|c| range(c, c)));
    sorted.sort_by_key(|r| r.from);
    let mut inv_ranges = Vec::new();
    let mut inv_chars = Vec::new();
    let mut start = all.from;
    for r in sorted {
        if start <= r.from {
            if start + 1 == r.from {
                inv_chars.push(start as u16);
            } else if start != r.from {
                inv_ranges.push(range(start, r.from - 1));
            }
        }
        start = r.to + 1;
        if start > all.to {
            return (inv_ranges, inv_chars);
        }
    }
    if start < all.to {
        inv_ranges.push(range(start, all.to));
    } else if start == all.to {
        inv_chars.push(start as u16);
    }
    (inv_ranges, inv_chars)
}

#[derive(Debug, Clone)]
enum Kind {
    Final(Str),
    Set(Box<SymbolSet>),
    Choice(Vec<Node>),
    Sequence(Vec<Node>),
    Repeat(Box<Node>, i32, i32),
    Group(usize, Box<Node>),
    GroupRef(usize),
    /// A negative lookaround. Its pattern is parsed again when `NotMatchingGenerationVisitor` meets it nested in
    /// another one.
    Not(Box<Node>),
}

#[derive(Debug, Clone)]
struct Node {
    id: usize,
    pattern: Str,
    kind: Kind,
}

fn s(text: &str) -> Str {
    text.encode_utf16().collect()
}

fn err(class: &'static str, message: String) -> Error {
    Error::new(class, message)
}

fn parse_error(message: String) -> Error {
    err("RgxGenParseException", message)
}

/// `CharIterator`.
struct CharIterator {
    value: Str,
    bound: usize,
    current: usize,
}

impl CharIterator {
    fn has_next(&self) -> bool {
        self.current < self.bound
    }

    fn skip(&mut self, n: usize) {
        self.current += n;
    }

    fn peek(&self, offset: isize) -> u16 {
        let pos = self.current as isize + offset;
        if pos < 0 || pos >= self.bound as isize { 0 } else { self.value[pos as usize] }
    }

    fn next(&mut self) -> Result<u16> {
        match self.value.get(self.current) {
            Some(c) => {
                self.current += 1;
                Ok(*c)
            }
            None => Err(err(
                "NoSuchElementException",
                format!("Index {} out of bounds for length {}", self.current, self.value.len()),
            )),
        }
    }

    fn next_n(&mut self, length: usize) -> Str {
        let up_to = (self.current + length).min(self.bound);
        let sub = self.value[self.current.min(up_to)..up_to].to_vec();
        self.current = up_to;
        sub
    }

    fn context_at(&self, index: isize) -> String {
        let start = (index - 5).max(0) as usize;
        let end = ((index + 5).max(0) as usize).min(self.bound);
        let offset = if start == 0 { index } else { 5 };
        let sub = if start <= end { String::from_utf16_lossy(&self.value[start..end]) } else { String::new() };
        let spaces = if 1 + offset < 0 { String::new() } else { " ".repeat((1 + offset) as usize) };
        format!("\n'{sub}'\n{spaces}^")
    }

    fn context(&self) -> String {
        self.context_at(self.current as isize - 1)
    }

    /// `String.indexOf(needle, from)`.
    fn index_of(&self, needle: &[u16], from: usize) -> isize {
        let n = self.value.len();
        if needle.len() > n {
            return -1;
        }
        (from..=n - needle.len()).find(|&i| self.value[i..i + needle.len()] == *needle).map_or(-1, |i| i as isize)
    }

    fn peek_until(&self, needle: &[u16], must_exist: bool) -> Result<Str> {
        let start = self.current;
        let mut tmp = self.current as isize;
        let len = needle.len() as isize;
        let end;
        loop {
            tmp = self.index_of(needle, tmp as usize);
            if tmp + len > self.bound as isize {
                end = self.bound;
                break;
            } else if tmp < 0 {
                if must_exist {
                    return Err(err(
                        "NoSuchElementException",
                        format!(
                            "Could not find termination sequence/character in string: '{}",
                            String::from_utf16_lossy(&self.value[start..])
                        ),
                    ));
                }
                end = self.bound;
                break;
            }
            let mut cnt = 1;
            loop {
                let at = tmp - cnt;
                if at < 0 {
                    return Err(err(
                        "StringIndexOutOfBoundsException",
                        format!("Index {at} out of bounds for length {}", self.value.len()),
                    ));
                }
                if self.value[at as usize] != b'\\' as u16 {
                    break;
                }
                cnt += 1;
            }
            if cnt % 2 != 0 {
                end = tmp as usize;
                break;
            }
            tmp += 1;
        }
        Ok(self.value[start..end].to_vec())
    }

    fn next_until(&mut self, needle: &[u16], must_exist: bool) -> Result<Str> {
        let peek = self.peek_until(needle, must_exist)?;
        self.current = (self.current + peek.len() + needle.len()).min(self.bound);
        Ok(peek)
    }

    fn take_while_digits(&mut self) -> Str {
        let start = self.current;
        while self.has_next() {
            let c = self.value[self.current];
            self.current += 1;
            if !crate::java::is_digit(c) {
                self.current -= 1;
                break;
            }
        }
        self.value[start..self.current].to_vec()
    }

    fn prev_pos(&self) -> isize {
        self.current as isize - 1
    }

    fn substring_to_curr(&self, pos: isize) -> Str {
        self.value[pos.max(0) as usize..self.current].to_vec()
    }

    fn has_next_str(&self, chars: &str) -> bool {
        chars.encode_utf16().enumerate().all(|(i, c)| self.peek(i as isize) == c)
    }
}

/// `Integer.parseInt(s, radix)`.
fn parse_int(text: &[u16], radix: u32) -> Result<i32> {
    let st = String::from_utf16_lossy(text);
    let nfe = || {
        let suffix = if radix == 10 { String::new() } else { format!(" under radix {radix}") };
        err("NumberFormatException", format!("For input string: \"{st}\"{suffix}"))
    };
    let (negative, digits) = match st.chars().next() {
        Some('-') => (true, &st[1..]),
        Some('+') => (false, &st[1..]),
        _ => (false, &st[..]),
    };
    if digits.is_empty() {
        return Err(nfe());
    }
    let mut value: i64 = 0;
    for ch in digits.chars() {
        let d = ch.to_digit(radix).ok_or_else(nfe)? as i64;
        value = value * radix as i64 + d;
        if value > i32::MAX as i64 + 1 {
            return Err(nfe());
        }
    }
    let value = if negative { -value } else { value };
    i32::try_from(value).map_err(|_| nfe())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupType {
    Lookaround,
    NegativeLookaround,
    Capture,
    NonCapture,
}

/// `DefaultTreeBuilder`. `starts` is its `aNodesStartPos`, which shapes the patterns nodes keep (a negative group
/// is parsed again from its pattern) and makes a quantified negative group fail as upstream's does.
struct TreeBuilder {
    it: CharIterator,
    starts: HashMap<usize, isize>,
    next_id: usize,
    next_group: usize,
}

impl TreeBuilder {
    fn node(&mut self, pattern: Str, kind: Kind) -> Node {
        self.next_id += 1;
        Node { id: self.next_id, pattern, kind }
    }

    fn final_symbol(&mut self, value: Str) -> Node {
        self.node(value.clone(), Kind::Final(value))
    }

    fn set_node(&mut self, set: SymbolSet) -> Node {
        let pattern = set.pattern.clone();
        self.node(pattern, Kind::Set(Box::new(set)))
    }

    fn sb_to_final(&mut self, sb: &mut Str, nodes: &mut Vec<Node>) {
        if !sb.is_empty() {
            let value = std::mem::take(sb);
            let len = value.len() as isize;
            let f = self.final_symbol(value);
            self.starts.insert(f.id, self.it.prev_pos() - len);
            nodes.push(f);
        }
    }

    fn process_group_type(&mut self) -> Result<GroupType> {
        let mut skip = 2;
        if self.it.peek(0) != b'?' as u16 {
            return Ok(GroupType::Capture);
        }
        let group_type = if self.it.has_next_str("?:") {
            GroupType::NonCapture
        } else if self.it.has_next_str("?=") {
            GroupType::Lookaround
        } else if self.it.has_next_str("?!") {
            GroupType::NegativeLookaround
        } else if self.it.has_next_str("?<=") {
            skip = 3;
            GroupType::Lookaround
        } else if self.it.has_next_str("?<!") {
            skip = 3;
            GroupType::NegativeLookaround
        } else if self.it.has_next_str("?<") {
            match self.it.peek_until(&s(">"), true) {
                Ok(name) => {
                    skip = name.len() + 1;
                    GroupType::Capture
                }
                Err(_) => {
                    self.it.skip(skip);
                    return Err(parse_error(format!("Incomplete group structure: {}", self.it.context())));
                }
            }
        } else {
            self.it.skip(skip);
            return Err(parse_error(format!("Incomplete group structure: {}", self.it.context())));
        };
        self.it.skip(skip);
        Ok(group_type)
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_group_end(
        &mut self,
        start: isize,
        sb: &mut Str,
        nodes: &mut Vec<Node>,
        is_choice: bool,
        choices: &mut Vec<Node>,
        capture: Option<usize>,
        group_type: GroupType,
    ) -> Result<Node> {
        if sb.is_empty() && nodes.is_empty() {
            let f = self.final_symbol(Vec::new());
            self.starts.insert(f.id, start);
            nodes.push(f);
        } else {
            self.sb_to_final(sb, nodes);
        }
        if is_choice {
            let n = self.sequence_or_not(start, nodes, choices, false, None)?;
            choices.push(n);
            nodes.clear();
        }
        let node = self.sequence_or_not(start, nodes, choices, is_choice, capture)?;
        if group_type == GroupType::NegativeLookaround {
            let pattern = node.pattern.clone();
            Ok(self.node(pattern, Kind::Not(Box::new(node))))
        } else {
            Ok(node)
        }
    }

    fn verify_start_end(&self, c: u16) -> Result<()> {
        let caret = c == b'^' as u16;
        let at = self.it.peek(if caret { -2 } else { 0 });
        let text = match char::from_u32(at as u32).unwrap_or('\u{FFFD}') {
            '+' | '*' | '{' | '?' | '\0' | '\n' | '\r' | '|' => return Ok(()),
            '^' | '$' => "Start and end of line markers cannot be put together.",
            '(' if !caret => "After dollar only new line is allowed!",
            '(' => return Ok(()),
            ')' if caret => "Before caret only new line is allowed!",
            ')' => return Ok(()),
            _ if caret => "Before caret only new line is allowed!",
            _ => "After dollar only new line is allowed!",
        };
        Err(err("PatternDoesNotMatchAnythingException", format!("{text}{}", self.it.context())))
    }

    fn parse_group(&mut self, group_start: isize, group_type: GroupType) -> Result<Node> {
        let capture = if group_type == GroupType::Capture {
            self.next_group += 1;
            Some(self.next_group - 1)
        } else {
            None
        };
        let mut choices: Vec<Node> = Vec::new();
        let mut nodes: Vec<Node> = Vec::new();
        let mut sb: Str = Vec::new();
        let mut is_choice = false;
        let mut choices_start = group_start;
        while self.it.has_next() {
            let c = self.it.next()?;
            match char::from_u32(c as u32).unwrap_or('\u{FFFD}') {
                '^' | '$' => self.verify_start_end(c)?,
                '[' => {
                    self.sb_to_final(&mut sb, &mut nodes);
                    let n = self.square_brackets()?;
                    nodes.push(n);
                }
                '(' => {
                    self.sb_to_final(&mut sb, &mut nodes);
                    let start = self.it.prev_pos();
                    let t = self.process_group_type()?;
                    let n = self.parse_group(start, t)?;
                    nodes.push(n);
                }
                '|' => {
                    if sb.is_empty() && nodes.is_empty() {
                        let f = self.final_symbol(Vec::new());
                        self.starts.insert(f.id, self.it.prev_pos() + 1);
                        choices.push(f);
                    } else {
                        self.sb_to_final(&mut sb, &mut nodes);
                        let n = self.sequence_or_not(choices_start, &mut nodes, &mut choices, false, None)?;
                        choices.push(n);
                        choices_start = self.it.prev_pos() + 1;
                        nodes.clear();
                    }
                    is_choice = true;
                }
                ')' => {
                    return self.handle_group_end(
                        group_start,
                        &mut sb,
                        &mut nodes,
                        is_choice,
                        &mut choices,
                        capture,
                        group_type,
                    );
                }
                '{' | '*' | '?' | '+' => self.repeat_character(&mut nodes, &mut sb, c)?,
                '.' => {
                    self.sb_to_final(&mut sb, &mut nodes);
                    let set = SymbolSet::ascii(".", vec![ASCII_SYMBOL_RANGE], Vec::new(), MatchType::Positive);
                    let n = self.set_node(set);
                    self.starts.insert(n.id, self.it.prev_pos());
                    nodes.push(n);
                }
                '\\' => self.escaped(&mut sb, &mut nodes, true)?,
                _ => sb.push(c),
            }
        }
        self.handle_group_end(group_start, &mut sb, &mut nodes, is_choice, &mut choices, capture, group_type)
    }

    fn repeat_character(&mut self, nodes: &mut Vec<Node>, sb: &mut Str, c: u16) -> Result<()> {
        let repeat_node = if sb.is_empty() {
            match nodes.pop() {
                Some(n) => n,
                None => {
                    let prev = self.it.peek(-2);
                    if prev == b'^' as u16 || prev == b'$' as u16 {
                        return Err(err(
                            "TokenNotQuantifiableException",
                            format!("{} at {}", String::from_utf16_lossy(&[prev]), self.it.context()),
                        ));
                    }
                    return Err(parse_error(format!("Cannot repeat nothing at{}", self.it.context())));
                }
            }
        } else {
            let last = sb.pop().expect("non-empty");
            self.sb_to_final(sb, nodes);
            let n = self.final_symbol(vec![last]);
            self.starts.insert(n.id, self.it.prev_pos() - 1);
            n
        };
        let n = self.handle_repeat(c, repeat_node)?;
        nodes.push(n);
        Ok(())
    }

    fn handle_repeat(&mut self, c: u16, repeat_node: Node) -> Result<Node> {
        let Some(&start) = self.starts.get(&repeat_node.id) else {
            return Err(err(
                "NullPointerException",
                "Cannot invoke \"java.lang.Integer.intValue()\" because the return value of \"java.util.Map.get(Object)\" is null"
                    .into(),
            ));
        };
        let node = match char::from_u32(c as u32).unwrap_or('\u{FFFD}') {
            '*' => self.repeat(start, repeat_node, 0, -1),
            '?' => self.repeat(start, repeat_node, 0, 1),
            '+' => self.repeat(start, repeat_node, 1, -1),
            _ => self.repeat_in_braces(start, repeat_node)?,
        };
        self.starts.insert(node.id, start);
        Ok(node)
    }

    fn repeat(&mut self, start: isize, node: Node, min: i32, max: i32) -> Node {
        let pattern = self.it.substring_to_curr(start);
        self.node(pattern, Kind::Repeat(Box::new(node), min, max))
    }

    fn repeat_in_braces(&mut self, start: isize, node: Node) -> Result<Node> {
        let mut sb: Str = Vec::new();
        let mut min = -1;
        let context_index = self.it.prev_pos();
        while self.it.has_next() {
            let c = self.it.next()?;
            if c == b',' as u16 {
                let tmp_context = self.it.prev_pos() - 1;
                min = parse_int(&sb, 10).map_err(|_| {
                    parse_error(format!("Malformed lower bound number.{}", self.it.context_at(tmp_context)))
                })?;
                sb.clear();
            } else if c == b'}' as u16 {
                if min == -1 {
                    let n = parse_int(&sb, 10)?;
                    return Ok(self.repeat(start, node, n, n));
                } else if sb.is_empty() {
                    return Ok(self.repeat(start, node, min, -1));
                } else {
                    let max = parse_int(&sb, 10)
                        .map_err(|_| parse_error(format!("Malformed upper bound number.{}", self.it.context())))?;
                    return Ok(self.repeat(start, node, min, max));
                }
            } else if c == b'\\' as u16 {
                return Err(parse_error(format!(
                    "Escape character inside curvy repetition is not supported. {}",
                    self.it.context()
                )));
            } else {
                sb.push(c);
            }
        }
        Err(parse_error(format!("Unbalanced '{{' - missing '}}' at {}", self.it.context_at(context_index))))
    }

    fn sequence_or_not(
        &mut self,
        start: isize,
        nodes: &mut Vec<Node>,
        choices: &mut Vec<Node>,
        is_choice: bool,
        capture: Option<usize>,
    ) -> Result<Node> {
        let result = if nodes.len() == 1 {
            nodes.remove(0)
        } else if is_choice {
            if choices.is_empty() {
                return Err(parse_error("Empty nodes".into()));
            }
            let pattern = self.it.substring_to_curr(start);
            let alternatives = std::mem::take(choices);
            self.node(pattern, Kind::Choice(alternatives))
        } else {
            if nodes.is_empty() {
                return Err(parse_error("Empty nodes".into()));
            }
            let pattern = self.it.substring_to_curr(start);
            let items = std::mem::take(nodes);
            self.node(pattern, Kind::Sequence(items))
        };
        self.starts.insert(result.id, start);
        match capture {
            None => Ok(result),
            Some(index) => {
                let pattern = self.it.substring_to_curr(start);
                let group = self.node(pattern, Kind::Group(index, Box::new(result)));
                self.starts.insert(group.id, start);
                Ok(group)
            }
        }
    }

    /// `handleEscapedCharacter`.
    fn escaped(&mut self, sb: &mut Str, nodes: &mut Vec<Node>, group_ref_allowed: bool) -> Result<()> {
        let c = self.it.next()?;
        let start = self.it.prev_pos() - 1;
        let ch = char::from_u32(c as u32).unwrap_or('\u{FFFD}');
        let pattern = format!("\\{ch}");
        let match_type = |p: char| if ch == p { MatchType::Positive } else { MatchType::Negative };
        let created = match ch {
            'd' | 'D' => {
                self.sb_to_final(sb, nodes);
                Some(SymbolSet::ascii(&pattern, vec![range('0' as i32, '9' as i32)], Vec::new(), match_type('d')))
            }
            's' | 'S' => {
                self.sb_to_final(sb, nodes);
                let positive = CharsDefinition { ranges: Vec::new(), chars: s(" \t") };
                let negative = CharsDefinition { ranges: Vec::new(), chars: s("\t\n\u{000B}\u{000C}\r ") };
                Some(SymbolSet::new(s(&pattern), positive, Some(negative), match_type('s'), ASCII_SYMBOL_RANGE))
            }
            'w' | 'W' => {
                self.sb_to_final(sb, nodes);
                let ranges =
                    vec![range('a' as i32, 'z' as i32), range('A' as i32, 'Z' as i32), range('0' as i32, '9' as i32)];
                Some(SymbolSet::ascii(&pattern, ranges, s("_"), match_type('w')))
            }
            'p' | 'P' => {
                self.sb_to_final(sb, nodes);
                let key = if self.it.peek(0) == b'{' as u16 {
                    self.it.skip(1);
                    self.it.next_until(&s("}"), true)?
                } else {
                    self.it.next_n(1)
                };
                let key_text = String::from_utf16_lossy(&key);
                let Some(category) = unicode::category(&key_text) else {
                    return Err(err(
                        "NullPointerException",
                        "Cannot invoke \"net.datafaker.shaded.curiousoddman.rgxgen.model.UnicodeCategory.getSymbolRanges()\" because \"unicodeCategory\" is null"
                            .into(),
                    ));
                };
                let defs = CharsDefinition {
                    ranges: category.ranges.iter().map(|(f, t)| range(*f as i32, *t as i32)).collect(),
                    chars: category.symbols.to_vec(),
                };
                let pattern = s(&format!("\\{ch}{{{key_text}}}"));
                Some(SymbolSet::new(pattern, defs, None, match_type('p'), UNICODE_SYMBOL_RANGE))
            }
            'x' => {
                let hex = if self.it.peek(0) == b'{' as u16 {
                    self.it.skip(1);
                    self.it.next_until(&s("}"), true)?
                } else {
                    self.it.next_n(2)
                };
                sb.push(parse_int(&hex, 16)? as u16);
                None
            }
            'u' => {
                let hex = self.it.next_n(4);
                sb.push(parse_int(&hex, 16)? as u16);
                None
            }
            'Q' => {
                let quoted = self.it.next_until(&s("\\E"), false)?;
                sb.extend(quoted);
                None
            }
            'E' | 'b' | 'B' => None,
            '1'..='9' => {
                self.sb_to_final(sb, nodes);
                if !group_ref_allowed {
                    return Err(parse_error(format!("Group ref is not expected here. {}", self.it.context())));
                }
                let start = self.it.prev_pos() - 1;
                let mut number = vec![c];
                number.extend(self.it.take_while_digits());
                let index = parse_int(&number, 10)?;
                let mut pattern = s("\\");
                pattern.extend_from_slice(&number);
                let n = self.node(pattern, Kind::GroupRef(index as usize));
                self.starts.insert(n.id, start);
                nodes.push(n);
                None
            }
            _ => {
                sb.push(c);
                None
            }
        };
        if let Some(set) = created {
            let n = self.set_node(set);
            self.starts.insert(n.id, start);
            nodes.push(n);
        }
        Ok(())
    }

    fn square_brackets(&mut self) -> Result<Node> {
        let open = self.it.prev_pos();
        let match_type = if self.it.peek(0) == b'^' as u16 {
            self.it.skip(1);
            MatchType::Negative
        } else {
            MatchType::Positive
        };
        let mut characters: Str = Vec::new();
        let mut ranges: Vec<SymbolRange> = Vec::new();
        let mut sets: Vec<SymbolSet> = Vec::new();
        let mut range_started = false;
        let symbol_range = |characters: &mut Str, ranges: &mut Vec<SymbolRange>| {
            if characters.len() < 2 {
                characters.push(b'-' as u16);
            } else {
                let last = characters.pop().expect("two") as i32;
                let first = characters.pop().expect("two") as i32;
                ranges.push(range(first, last));
            }
        };
        while self.it.has_next() {
            let c = self.it.next()?;
            if c == b']' as u16 {
                let pattern = self.it.substring_to_curr(open);
                let set = symbol_set_from_brackets(pattern, match_type, characters, ranges, sets);
                let n = self.set_node(set);
                self.starts.insert(n.id, open);
                return Ok(n);
            } else if c == b'-' as u16 {
                if self.it.peek(0) == b']' as u16 || self.it.peek(-2) == b'[' as u16 {
                    characters.push(c);
                } else {
                    range_started = true;
                }
            } else if c == b'\\' as u16 {
                let mut nodes = Vec::new();
                let mut sb = Vec::new();
                self.escaped(&mut sb, &mut nodes, false)?;
                characters.extend(sb);
                if nodes.len() > 1 {
                    return Err(parse_error(format!(
                        "Multiple nodes found inside square brackets escape sequence before '{}'",
                        self.it.context()
                    )));
                }
                let set = nodes.pop().and_then(|n| match n.kind {
                    Kind::Set(set) => Some(*set),
                    _ => None,
                });
                if range_started {
                    if set.is_some() {
                        return Err(parse_error(format!(
                            "Cannot make range with a shorthand escape sequences before '{}'",
                            self.it.context()
                        )));
                    }
                    symbol_range(&mut characters, &mut ranges);
                } else if let Some(set) = set {
                    sets.push(set);
                }
                range_started = false;
            } else {
                characters.push(c);
                if range_started {
                    symbol_range(&mut characters, &mut ranges);
                    range_started = false;
                }
            }
        }
        Err(parse_error(format!("Unexpected End Of Expression. Didn't find closing ']'{}", self.it.context_at(open))))
    }
}

/// `createSymbolSetFromSquareBrackets`.
fn symbol_set_from_brackets(
    pattern: Str,
    match_type: MatchType,
    characters: Str,
    ranges: Vec<SymbolRange>,
    sets: Vec<SymbolSet>,
) -> SymbolSet {
    let mut positive = CharsDefinition { ranges, chars: characters };
    let modified = sets.iter().any(|s| s.negative_exclusion.is_some());
    let mut negative = if modified { Some(positive.clone()) } else { None };
    let mut is_ascii = true;
    for set in &sets {
        is_ascii = is_ascii && set.is_ascii;
        positive.chars.extend_from_slice(&set.symbols);
        positive.ranges.extend_from_slice(&set.ranges);
        if let Some(neg) = negative.as_mut() {
            match &set.negative_exclusion {
                Some(ex) => {
                    neg.ranges.extend_from_slice(&ex.ranges);
                    neg.chars.extend_from_slice(&ex.chars);
                }
                None => {
                    neg.chars.extend_from_slice(&set.symbols);
                    neg.ranges.extend_from_slice(&set.ranges);
                }
            }
        }
    }
    let universe = if is_ascii { ASCII_SYMBOL_RANGE } else { UNICODE_SYMBOL_RANGE };
    SymbolSet::new(pattern, positive, negative, match_type, universe)
}

fn parse_tree(pattern: &[u16]) -> Result<Node> {
    let mut b = TreeBuilder {
        it: CharIterator { value: pattern.to_vec(), bound: pattern.len(), current: 0 },
        starts: HashMap::new(),
        next_id: 0,
        next_group: 1,
    };
    let node = b.parse_group(0, GroupType::NonCapture)?;
    if b.it.has_next() {
        return Err(parse_error(format!("Expression was not fully parsed: {}", b.it.context())));
    }
    Ok(node)
}

/// How deep negative groups may nest through re-parsing before giving up (upstream recurses until
/// `StackOverflowError` on a pattern that re-parses into itself).
const MAX_NOT_DEPTH: usize = 32;

/// Whether generating `node` can reach `NotMatchingGenerationVisitor.visit(Choice)` (`matching` false inside a
/// negative group), which needs `java.util.regex`, or nests negative groups too deeply.
fn needs_regex_matching(node: &Node, matching: bool, depth: usize) -> bool {
    match &node.kind {
        Kind::Final(_) | Kind::Set(_) | Kind::GroupRef(_) => false,
        Kind::Choice(alternatives) => !matching || alternatives.iter().any(|n| needs_regex_matching(n, matching, depth)),
        Kind::Sequence(items) => items.iter().any(|n| needs_regex_matching(n, matching, depth)),
        Kind::Repeat(n, _, _) | Kind::Group(_, n) => needs_regex_matching(n, matching, depth),
        Kind::Not(inner) => {
            if depth >= MAX_NOT_DEPTH {
                return true;
            }
            if matching {
                needs_regex_matching(inner, false, depth + 1)
            } else {
                parse_tree(&node.pattern).map_or(true, |r| needs_regex_matching(&r, true, depth + 1))
            }
        }
    }
}

/// A parsed pattern (`RgxGen.parse(pattern)`).
#[derive(Debug, Clone)]
pub struct RgxGen {
    node: Node,
}

impl RgxGen {
    /// `RgxGen.parse(pattern)`.
    pub fn parse(pattern: &str) -> Result<RgxGen> {
        let node = parse_tree(&s(pattern))?;
        if needs_regex_matching(&node, true, 0) {
            return Err(Error::new(
                "UnsupportedOperationException",
                "a negative lookaround with an alternation needs java.util.regex, which is not ported",
            ));
        }
        Ok(RgxGen { node })
    }

    /// `generate(random)`. Generation can fail as RgxGen's does (an empty character class is `nextInt(0)`); this
    /// returns an empty string then, and [`RgxGen::try_generate`] the exception.
    pub fn generate(&self, random: &dyn RandomGenerator) -> String {
        self.try_generate(random).unwrap_or_default()
    }

    /// `generate(random)`, with the exception RgxGen would throw.
    pub fn try_generate(&self, random: &dyn RandomGenerator) -> Result<String> {
        let mut groups = HashMap::new();
        let mut out = Vec::new();
        visit(&self.node, random, &mut groups, &mut out, true)?;
        Ok(String::from_utf16_lossy(&out))
    }
}

/// `GenerationVisitor` (`matching`) and `NotMatchingGenerationVisitor` (not `matching`).
fn visit(node: &Node, random: &dyn RandomGenerator, groups: &mut HashMap<usize, Str>, out: &mut Str, matching: bool) -> Result<()> {
    match &node.kind {
        Kind::Set(set) => {
            if matching {
                let idx = random.next_int_bounded(set.size)?;
                out.push(set.get(idx));
            } else {
                let inverted = set.inverted();
                if inverted.size != 0 {
                    let idx = random.next_int_bounded(inverted.size)?;
                    out.push(inverted.get(idx));
                }
            }
        }
        Kind::Choice(nodes) => {
            if !matching {
                return Err(Error::new("UnsupportedOperationException", "NotMatchingGenerationVisitor.visit(Choice)"));
            }
            let idx = random.next_int_bounded(nodes.len() as i32)?;
            visit(&nodes[idx as usize], random, groups, out, matching)?;
        }
        Kind::Final(value) => {
            if matching {
                out.extend_from_slice(value);
            } else {
                not_matching_final(value, random, out)?;
            }
        }
        Kind::Repeat(inner, min, max) => {
            let min = if !matching && *min == 0 { 1 } else { *min };
            let max = if *max == -1 { INFINITE_PATTERN_REPETITION } else { *max };
            let repeat = if min >= max { min } else { min + random.next_int_bounded(max + 1 - min)? };
            for _ in 0..repeat {
                visit(inner, random, groups, out, matching)?;
            }
        }
        Kind::Sequence(nodes) => {
            for n in nodes {
                visit(n, random, groups, out, matching)?;
            }
        }
        Kind::Not(inner) => {
            let mut sub = Vec::new();
            if matching {
                visit(inner, random, groups, &mut sub, false)?;
            } else {
                let reparsed = parse_tree(&node.pattern)?;
                visit(&reparsed, random, groups, &mut sub, true)?;
            }
            out.extend(sub);
        }
        Kind::GroupRef(index) => {
            if matching {
                out.extend_from_slice(groups.get(index).map_or(&[][..], |v| &v[..]));
            } else {
                let value = groups.get(index).cloned().ok_or_else(|| {
                    Error::new("NullPointerException", "Cannot invoke \"String.isEmpty()\" because \"nodeValue\" is null")
                })?;
                not_matching_final(&value, random, out)?;
            }
        }
        Kind::Group(index, inner) => {
            let start = out.len();
            visit(inner, random, groups, out, matching)?;
            groups.insert(*index, out[start..].to_vec());
        }
    }
    Ok(())
}

/// `NotMatchingGenerationVisitor.visit(FinalSymbol)`: random printable ASCII, redrawn while equal to the value.
fn not_matching_final(value: &[u16], random: &dyn RandomGenerator, out: &mut Str) -> Result<()> {
    let any = |random: &dyn RandomGenerator| -> Result<u16> {
        Ok((ASCII_SYMBOL_RANGE.from + random.next_int_bounded(ASCII_SYMBOL_RANGE.size())?) as u16)
    };
    if value.is_empty() {
        out.push(any(random)?);
        return Ok(());
    }
    loop {
        let mut built = Vec::with_capacity(value.len());
        for _ in value {
            built.push(any(random)?);
        }
        if built != value {
            out.extend(built);
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_patterns() {
        let r = crate::java::Random::new(1);
        let g = RgxGen::parse("[a-c]{3}x").unwrap().generate(&r);
        assert_eq!(g.len(), 4);
        assert!(g.ends_with('x'));
        assert_eq!(RgxGen::parse("a{").unwrap_err().simple_class(), "RgxGenParseException");
    }
}
