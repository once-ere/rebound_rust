//! integrators_test.rs — Rust twin of porttest/integrators_test.c.
//! Part of rebound_rs, GPL-3.0-or-later.

use rebound_rs::*;
use std::io::Write;

fn bits(x: f64) -> u64 {
    x.to_bits()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let integrator = if args.len() > 1 { args[1].clone() } else { "ias15".to_string() };
    let order: u32 = if args.len() > 2 { args[2].parse().unwrap_or(2) } else { 2 };
    let nsteps: usize = if args.len() > 3 { args[3].parse().unwrap_or(1000) } else { 1000 };

    let mut sim = reb_simulation_create();
    let r = &mut sim;
    reb_simulation_set_integrator(r, &integrator);
    if integrator == "leapfrog" {
        if let reb_integrator_state::leapfrog(ref mut lf) = r.integrator {
            lf.order = order;
        }
    }
    r.G = 1.0;
    r.dt = 0.01;

    let mut star = reb_particle::default();
    star.m = 1.0;
    reb_simulation_add(r, star);

    let mut planet = reb_particle::default();
    planet.m = 1e-3;
    planet.x = 1.6; // apocenter of a=1, e=0.6 orbit
    planet.vy = 0.5; // roughly the apocenter speed
    reb_simulation_add(r, planet);

    let mut moon = reb_particle::default();
    moon.m = 1e-7;
    moon.x = 1.7;
    moon.vy = 0.6;
    moon.z = 0.01;
    moon.vz = 0.001;
    reb_simulation_add(r, moon);

    reb_simulation_steps(r, nsteps);

    let mut f = std::fs::File::create("state_rust_final.txt").unwrap();
    writeln!(f, "integrator {} order {} steps {}", integrator, order, nsteps).unwrap();
    writeln!(f, "t {:016x}", bits(r.t)).unwrap();
    writeln!(f, "dt {:016x}", bits(r.dt)).unwrap();
    writeln!(f, "steps_done {}", r.steps_done).unwrap();
    for i in 0..r.N {
        let p = r.particles[i];
        writeln!(
            f,
            "{} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
            i,
            bits(p.x),
            bits(p.y),
            bits(p.z),
            bits(p.vx),
            bits(p.vy),
            bits(p.vz)
        )
        .unwrap();
    }
    drop(f);
    println!("{} done: t={:e} steps={}", integrator, r.t, r.steps_done);
}
