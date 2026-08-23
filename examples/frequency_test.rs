//! frequency_test.rs — Rust twin of porttest/frequency_test.c.
//! Part of rebound_rs, GPL-3.0-or-later.

use rebound_rs::frequency_analysis::*;
use std::io::Write;

fn bits(x: f64) -> u64 {
    x.to_bits()
}

fn main() {
    let ndata: usize = 256;
    let nfreq: usize = 3;
    let mut input = vec![0.0_f64; 2 * ndata];
    // Quasi-periodic signal with three frequencies (rad per sample).
    let (f1, a1, p1) = (0.30_f64, 1.00_f64, 0.40_f64);
    let (f2, a2, p2) = (0.55_f64, 0.35_f64, 1.90_f64);
    let (f3, a3, p3) = (0.11_f64, 0.10_f64, 5.10_f64);
    for i in 0..ndata {
        let t = i as f64;
        input[2 * i] = a1 * (f1 * t + p1).cos() + a2 * (f2 * t + p2).cos() + a3 * (f3 * t + p3).cos();
        input[2 * i + 1] =
            a1 * (f1 * t + p1).sin() + a2 * (f2 * t + p2).sin() + a3 * (f3 * t + p3).sin();
    }

    let mut f = std::fs::File::create("frequency_rust.txt").unwrap();
    let types = [
        REB_FREQUENCY_ANALYSIS_MFT,
        REB_FREQUENCY_ANALYSIS_FMFT,
        REB_FREQUENCY_ANALYSIS_FMFT2,
    ];
    let names = ["MFT", "FMFT", "FMFT2"];
    for ti in 0..3 {
        let mut output = [0.0_f64; 9];
        let ret = reb_frequency_analysis(&mut output, nfreq, 0.05, 1.0, types[ti], &input, ndata);
        writeln!(f, "{} ret {}", names[ti], ret).unwrap();
        for k in 0..3 * nfreq {
            writeln!(f, "{} {:016x}", k, bits(output[k])).unwrap();
        }
    }
    println!("frequency_test done");
}
