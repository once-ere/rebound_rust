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
    // whfast configurations are encoded as pseudo names; see the C twin.
    let real_integrator = if integrator.starts_with("whfast") { "whfast" } else { integrator.as_str() };
    reb_simulation_set_integrator(r, real_integrator);
    if integrator == "leapfrog" {
        if let reb_integrator_state::leapfrog(ref mut lf) = r.integrator {
            lf.order = order;
        }
    }
    if integrator.starts_with("whfast") {
        if let reb_integrator_state::whfast(ref mut wh) = r.integrator {
            match integrator.as_str() {
                "whfast-c11" => wh.corrector = 11,
                "whfast-c17" => {
                    wh.corrector = 17;
                    wh.corrector2 = 1;
                }
                "whfast-dh" => {
                    wh.coordinates =
                        rebound_rs::integrator_whfast::REB_INTEGRATOR_WHFAST_COORDINATES_DEMOCRATICHELIOCENTRIC
                }
                "whfast-whds" => {
                    wh.coordinates =
                        rebound_rs::integrator_whfast::REB_INTEGRATOR_WHFAST_COORDINATES_WHDS
                }
                "whfast-bary" => {
                    wh.coordinates =
                        rebound_rs::integrator_whfast::REB_INTEGRATOR_WHFAST_COORDINATES_BARYCENTRIC
                }
                "whfast-mk" => {
                    wh.kernel =
                        rebound_rs::integrator_whfast::REB_INTEGRATOR_WHFAST_KERNEL_MODIFIEDKICK
                }
                "whfast-comp" => {
                    wh.kernel =
                        rebound_rs::integrator_whfast::REB_INTEGRATOR_WHFAST_KERNEL_COMPOSITION
                }
                "whfast-lazy" => {
                    wh.kernel = rebound_rs::integrator_whfast::REB_INTEGRATOR_WHFAST_KERNEL_LAZY
                }
                "whfast-usafe" => wh.safe_mode = 0,
                _ => {}
            }
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
