//! addfmt_test.rs — Rust twin of porttest/addfmt_test.c.
//! Part of rebound_rs, GPL-3.0-or-later.
#![allow(non_snake_case)]

use rebound_rs::*;
use std::io::Write;

fn bits(x: f64) -> u64 {
    x.to_bits()
}

fn main() {
    let mut sim = reb_simulation_create();
    let r = &mut sim;
    r.G = 1.0;
    reb_simulation_add_fmt(r, "solar system", &[]);
    reb_simulation_add_fmt(
        r,
        "m a e inc Omega omega f",
        &[
            reb_fmt_arg::d(1e-9),
            reb_fmt_arg::d(12.5),
            reb_fmt_arg::d(0.3),
            reb_fmt_arg::d(0.2),
            reb_fmt_arg::d(0.6),
            reb_fmt_arg::d(1.1),
            reb_fmt_arg::d(2.5),
        ],
    );
    reb_simulation_add_fmt(
        r,
        "m a l h k ix iy",
        &[
            reb_fmt_arg::d(2e-9),
            reb_fmt_arg::d(15.5),
            reb_fmt_arg::d(0.7),
            reb_fmt_arg::d(0.05),
            reb_fmt_arg::d(-0.03),
            reb_fmt_arg::d(0.01),
            reb_fmt_arg::d(0.02),
        ],
    );
    reb_simulation_add_fmt(
        r,
        "m P e M",
        &[
            reb_fmt_arg::d(3e-9),
            reb_fmt_arg::d(100.0),
            reb_fmt_arg::d(0.1),
            reb_fmt_arg::d(0.5),
        ],
    );

    let mut f = std::fs::File::create("addfmt_rust.txt").unwrap();
    for i in 0..r.N {
        let p = r.particles[i];
        writeln!(
            f,
            "{} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
            i,
            bits(p.m),
            bits(p.x),
            bits(p.y),
            bits(p.z),
            bits(p.vx),
            bits(p.vy),
            bits(p.vz)
        )
        .unwrap();
    }
    println!("addfmt done N={}", r.N);
}
