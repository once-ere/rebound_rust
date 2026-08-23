//! libm_diff.rs — Rust twin of porttest/libm_diff.c. Same xorshift
//! corpus, same functions, bit-pattern output for diffing.
use std::io::Write;

fn bits(x: f64) -> u64 {
    x.to_bits()
}

struct Xs(u64);
impl Xs {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn main() {
    let mut s = Xs(88172645463325252u64);
    let mut f = std::io::BufWriter::new(std::fs::File::create("libm_rust.txt").unwrap());
    for _ in 0..200000 {
        let x = ((s.next() % 2000000000u64) as f64) / 1e6 - 1000.0;
        let y = ((s.next() % 2000000000u64) as f64) / 1e6 - 1000.0;
        let xp = x.abs() + 1e-9;
        writeln!(
            f,
            "{:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
            bits(x.sin()),
            bits(x.cos()),
            bits(x.tan()),
            bits(y.atan2(x)),
            bits(xp.powf(-0.234)),
            bits(xp.sqrt()),
            bits(y % 3.7),
            bits((x / 100.).exp()),
            bits(xp.ln())
        )
        .unwrap();
    }
    println!("done");
}
