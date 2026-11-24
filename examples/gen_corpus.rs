//! Deterministic corpus generator: `gen_corpus <out> <mixed|array|ndjson> <MiB>`.
//! Mix: ~40% short ASCII strings, 10% strings with escapes/unicode,
//! 25% ints, 15% floats, 10% bools/null; objects 3–12 keys, depth ≤ 6.

use std::io::{BufWriter, Write};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn value(r: &mut Rng, w: &mut impl Write, depth: u32) -> std::io::Result<()> {
    match (depth < 6).then(|| r.below(10)).unwrap_or(9) {
        0 => {
            write!(w, "{{")?;
            for i in 0..3 + r.below(10) {
                if i > 0 { write!(w, ",")?; }
                write!(w, "\"k{}\":", r.below(64))?;
                value(r, w, depth + 1)?;
            }
            write!(w, "}}")
        }
        1 => {
            write!(w, "[")?;
            for i in 0..r.below(8) {
                if i > 0 { write!(w, ",")?; }
                value(r, w, depth + 1)?;
            }
            write!(w, "]")
        }
        2 | 3 | 4 => write!(w, "\"s{:x}_{}\"", r.next(), "x".repeat(r.below(40) as usize)),
        5 => write!(w, r#""esc \"q\" \\ \n é {}""#, r.below(1000)),
        6 | 7 => write!(w, "{}", r.next() as i64 >> r.below(40)),
        8 => write!(w, "{}.{}e{}", r.below(1_000_000), r.below(1000), r.below(20) as i64 - 10),
        _ => write!(w, "{}", ["true", "false", "null"][r.below(3) as usize]),
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (out, kind, mib) = (&args[1], args[2].as_str(), args[3].parse::<u64>().unwrap());
    let target = mib << 20;
    let mut w = BufWriter::with_capacity(1 << 20, std::fs::File::create(out)?);
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    let mut written = 0u64;
    let counting = |w: &mut BufWriter<std::fs::File>| w.get_ref().metadata().map(|m| m.len());
    if kind != "ndjson" { write!(w, "[")?; }
    let mut first = true;
    while written < target {
        if kind == "ndjson" { value(&mut r, &mut w, 1)?; writeln!(w)?; }
        else { if !first { write!(w, ",")?; } first = false; value(&mut r, &mut w, 1)?; }
        if r.below(256) == 0 { w.flush()?; written = counting(&mut w)?; }
    }
    if kind != "ndjson" { write!(w, "]")?; }
    w.flush()
}
