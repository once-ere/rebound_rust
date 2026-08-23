//! server_test.rs — starts the Rust REBOUND webserver with a known
//! simulation state, pauses, and serves until a 'Q' key arrives via
//! HTTP (/keyboard/81). Used by the port verification to check that
//! the blob served at /simulation is a valid REBOUND binary that the
//! MSVC C build loads to the bit-identical state.
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
    reb_simulation_set_integrator(r, "whfast");
    r.G = 1.0;
    r.dt = 0.01;

    let mut star = reb_particle::default();
    star.m = 1.0;
    reb_simulation_add(r, star);
    let mut planet = reb_particle::default();
    planet.m = 1e-3;
    planet.x = 1.6;
    planet.vy = 0.5;
    reb_simulation_add(r, planet);
    let mut moon = reb_particle::default();
    moon.m = 1e-7;
    moon.x = 1.7;
    moon.vy = 0.6;
    moon.z = 0.01;
    moon.vz = 0.001;
    reb_simulation_add(r, moon);

    reb_simulation_steps(r, 100);

    // Dump the state the server is about to serve.
    let mut f = std::fs::File::create("server_state_rust.txt").unwrap();
    writeln!(f, "integrator whfast").unwrap();
    writeln!(f, "t {:016x}", bits(r.t)).unwrap();
    writeln!(f, "dt {:016x}", bits(r.dt)).unwrap();
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

    if reb_simulation_start_server(r, 12873) != 0 {
        println!("server failed to start");
        std::process::exit(1);
    }
    // Pause; the integrate loop's wait loop keeps serving snapshots and
    // applying keyboard commands until 'Q' (81) arrives.
    r.status = REB_STATUS_PAUSED;
    reb_simulation_integrate(r, r.t + 1.0);
    reb_simulation_stop_server(r);
    println!("server_test done");
}
