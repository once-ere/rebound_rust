//! tools.rs — random sampling, diagnostics and orbit conversions
//! (translated from tools.c and the rand_r shim in rebound.c).
//!
//! Part of rebound_rs, a GPL-3.0-or-later translation of REBOUND 5.1.1
//! (c) Hanno Rein, Shangfei Liu and contributors. See crate root.

use crate::types::*;

use std::f64::consts::PI as M_PI;

/// The value C's `RAND_MAX` takes in the Windows build of REBOUND
/// (tools.c: `#define REB_RAND_MAX 2147483647  // INT_MAX`). The C code
/// pairs its own glibc-style `rand_r` (31-bit output) with this
/// constant on every platform that matters here.
pub const REB_RAND_MAX: i32 = 2147483647;

/// glibc's `rand_r`, exactly as vendored in rebound.c for Windows
/// (three rounds of the 1103515245/12345 LCG, composing a 31-bit
/// result). All arithmetic is wrapping 32-bit unsigned, as in C.
pub fn rand_r(seed: &mut u32) -> i32 {
    let mut next: u32 = *seed;
    let mut result: i32;

    next = next.wrapping_mul(1103515245);
    next = next.wrapping_add(12345);
    result = ((next / 65536) % 2048) as i32;

    next = next.wrapping_mul(1103515245);
    next = next.wrapping_add(12345);
    result <<= 10;
    result ^= ((next / 65536) % 1024) as i32;

    next = next.wrapping_mul(1103515245);
    next = next.wrapping_add(12345);
    result <<= 10;
    result ^= ((next / 65536) % 1024) as i32;

    *seed = next;

    result
}

/// djb2 string hash (tools.c `reb_hash`).
pub fn reb_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for c in s.bytes() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_add(hash)
            .wrapping_add(c as u32);
    }
    hash
}

/// Time-and-pid based seed (tools.c `reb_tools_get_rand_seed`).
/// C: `tim.tv_usec + getpid()`.
pub fn reb_tools_get_rand_seed() -> u32 {
    let usec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_micros())
        .unwrap_or(0);
    let pid = std::process::id();
    usec.wrapping_add(pid)
}

/// tools.c `reb_random_uniform`. `r = None` reproduces the C NULL path
/// (a throwaway time-based seed).
pub fn reb_random_uniform(r: Option<&mut reb_simulation>, min: f64, max: f64) -> f64 {
    let mut local_seed;
    let seedp: &mut u32 = match r {
        Some(sim) => &mut sim.rand_seed,
        None => {
            local_seed = reb_tools_get_rand_seed();
            &mut local_seed
        }
    };
    (rand_r(seedp) as f64) / (REB_RAND_MAX as f64) * (max - min) + min
}

/// tools.c `reb_random_powerlaw`.
pub fn reb_random_powerlaw(r: Option<&mut reb_simulation>, min: f64, max: f64, slope: f64) -> f64 {
    let y = reb_random_uniform(r, 0., 1.);
    if slope == -1. {
        (y * (max / min).ln() + min.ln()).exp()
    } else {
        ((max.powf(slope + 1.) - min.powf(slope + 1.)) * y + min.powf(slope + 1.))
            .powf(1. / (slope + 1.))
    }
}

/// tools.c `reb_random_normal` (Marsaglia polar method, discarding the
/// second variate exactly like the C).
pub fn reb_random_normal(r: Option<&mut reb_simulation>, variance: f64) -> f64 {
    let mut v1 = 0.;
    let mut v2 = 0.;
    let mut rsq = 1.;
    let mut local_seed;
    let seedp: &mut u32 = match r {
        Some(sim) => &mut sim.rand_seed,
        None => {
            local_seed = reb_tools_get_rand_seed();
            &mut local_seed
        }
    };
    while rsq >= 1. || rsq < 1.0e-12 {
        v1 = 2. * (rand_r(seedp) as f64) / (REB_RAND_MAX as f64) - 1.0;
        v2 = 2. * (rand_r(seedp) as f64) / (REB_RAND_MAX as f64) - 1.0;
        rsq = v1 * v1 + v2 * v2;
    }
    let _ = v2;
    v1 * (-2. * rsq.ln() / rsq * variance).sqrt()
}

/// tools.c `reb_random_rayleigh`.
pub fn reb_random_rayleigh(r: Option<&mut reb_simulation>, sigma: f64) -> f64 {
    let y = reb_random_uniform(r, 0., 1.);
    sigma * (-2. * y.ln()).sqrt()
}

/// tools.c `reb_simulation_energy` (serial path).
pub fn reb_simulation_energy(r: &reb_simulation) -> f64 {
    let N = r.N;
    let N_active = if r.N_active == usize::MAX { N } else { r.N_active };
    let particles = &r.particles;
    let mut e_kin = 0.;
    let mut e_pot = 0.;
    let N_interact = if r.testparticle_type == 0 { N_active } else { N };
    for i in 0..N_interact {
        let pi = particles[i];
        e_kin += 0.5 * pi.m * (pi.vx * pi.vx + pi.vy * pi.vy + pi.vz * pi.vz);
    }
    for i in 0..N_active {
        let pi = particles[i];
        for j in (i + 1)..N_interact {
            let pj = particles[j];
            let dx = pi.x - pj.x;
            let dy = pi.y - pj.y;
            let dz = pi.z - pj.z;
            e_pot -= r.G * pj.m * pi.m / (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    e_kin + e_pot + r.energy_offset
}

/// tools.c `reb_simulation_angular_momentum`.
pub fn reb_simulation_angular_momentum(r: &reb_simulation) -> reb_vec3d {
    let mut L = reb_vec3d::default();
    for i in 0..r.N {
        let pi = r.particles[i];
        L.x += pi.m * (pi.y * pi.vz - pi.z * pi.vy);
        L.y += pi.m * (pi.z * pi.vx - pi.x * pi.vz);
        L.z += pi.m * (pi.x * pi.vy - pi.y * pi.vx);
    }
    L
}

/// tools.c `reb_simulation_move_to_hel`.
pub fn reb_simulation_move_to_hel(r: &mut reb_simulation) {
    let N = r.N;
    if N > 0 {
        let hel = r.particles[0];
        for i in 1..N {
            r.particles[i].x -= hel.x;
            r.particles[i].y -= hel.y;
            r.particles[i].z -= hel.z;
            r.particles[i].vx -= hel.vx;
            r.particles[i].vy -= hel.vy;
            r.particles[i].vz -= hel.vz;
        }
        r.particles[0].x = 0.;
        r.particles[0].y = 0.;
        r.particles[0].z = 0.;
        r.particles[0].vx = 0.;
        r.particles[0].vy = 0.;
        r.particles[0].vz = 0.;
    }
}

/// tools.c `reb_simulation_move_to_com` (variational branches carried;
/// they simply do not run while `var_config` is empty).
pub fn reb_simulation_move_to_com(r: &mut reb_simulation) {
    let com = reb_simulation_com(r);
    let N = r.N;

    // First order variational shift (2nd order not carried: the C block
    // is only reachable with 2nd-order variational particles, which are
    // added by derivatives-based workflows; a run that needs them gets
    // a clear error instead of silent wrong physics).
    for v in 0..r.var_config.len() {
        let vc = r.var_config[v];
        if vc.testparticle >= 0 {
            // Test particles do not affect the COM
        } else if vc.order == 2 {
            reb_simulation_error(
                r,
                "move_to_com with 2nd order variational particles is not carried in rebound_rs.",
            );
            return;
        } else if vc.order == 1 {
            let index = vc.index;
            let mut com_shift = reb_particle::default();
            let mut dm = 0.;
            for i in 0..N {
                dm += r.particles_var[i + index].m;
            }
            for i in 0..N {
                let p = r.particles[i];
                let pv = r.particles_var[i + index];
                com_shift.x += p.m / com.m * pv.x;
                com_shift.y += p.m / com.m * pv.y;
                com_shift.z += p.m / com.m * pv.z;
                com_shift.vx += p.m / com.m * pv.vx;
                com_shift.vy += p.m / com.m * pv.vy;
                com_shift.vz += p.m / com.m * pv.vz;

                com_shift.x += p.x / com.m * pv.m;
                com_shift.y += p.y / com.m * pv.m;
                com_shift.z += p.z / com.m * pv.m;
                com_shift.vx += p.vx / com.m * pv.m;
                com_shift.vy += p.vy / com.m * pv.m;
                com_shift.vz += p.vz / com.m * pv.m;

                com_shift.x -= p.x / (com.m * com.m) * p.m * dm;
                com_shift.y -= p.y / (com.m * com.m) * p.m * dm;
                com_shift.z -= p.z / (com.m * com.m) * p.m * dm;
                com_shift.vx -= p.vx / (com.m * com.m) * p.m * dm;
                com_shift.vy -= p.vy / (com.m * com.m) * p.m * dm;
                com_shift.vz -= p.vz / (com.m * com.m) * p.m * dm;
            }
            for i in 0..N {
                r.particles_var[i + index].x -= com_shift.x;
                r.particles_var[i + index].y -= com_shift.y;
                r.particles_var[i + index].z -= com_shift.z;
                r.particles_var[i + index].vx -= com_shift.vx;
                r.particles_var[i + index].vy -= com_shift.vy;
                r.particles_var[i + index].vz -= com_shift.vz;
            }
        }
    }

    // Finally do normal particles
    for i in 0..N {
        r.particles[i].x -= com.x;
        r.particles[i].y -= com.y;
        r.particles[i].z -= com.z;
        r.particles[i].vx -= com.vx;
        r.particles[i].vy -= com.vy;
        r.particles[i].vz -= com.vz;
    }

    crate::boundary::reb_boundary_check(r);
}

/// tools.c `reb_particle_com_of_pair`.
pub fn reb_particle_com_of_pair(mut p1: reb_particle, p2: reb_particle) -> reb_particle {
    p1.x = p1.x * p1.m + p2.x * p2.m;
    p1.y = p1.y * p1.m + p2.y * p2.m;
    p1.z = p1.z * p1.m + p2.z * p2.m;
    p1.vx = p1.vx * p1.m + p2.vx * p2.m;
    p1.vy = p1.vy * p1.m + p2.vy * p2.m;
    p1.vz = p1.vz * p1.m + p2.vz * p2.m;
    p1.ax = p1.ax * p1.m + p2.ax * p2.m;
    p1.ay = p1.ay * p1.m + p2.ay * p2.m;
    p1.az = p1.az * p1.m + p2.az * p2.m;

    p1.m += p2.m;
    if p1.m > 0. {
        p1.x /= p1.m;
        p1.y /= p1.m;
        p1.z /= p1.m;
        p1.vx /= p1.m;
        p1.vy /= p1.m;
        p1.vz /= p1.m;
        p1.ax /= p1.m;
        p1.ay /= p1.m;
        p1.az /= p1.m;
    }
    p1
}

/// tools.c `reb_simulation_com_range`.
pub fn reb_simulation_com_range(r: &reb_simulation, first: usize, last: usize) -> reb_particle {
    let mut com = reb_particle::default();
    for i in first..last {
        com = reb_particle_com_of_pair(com, r.particles[i]);
    }
    com
}

/// tools.c `reb_simulation_com` (serial path).
pub fn reb_simulation_com(r: &reb_simulation) -> reb_particle {
    reb_simulation_com_range(r, 0, r.N)
}

/// tools.c `reb_simulation_add_plummer`.
pub fn reb_simulation_add_plummer(r: &mut reb_simulation, _N: usize, M: f64, R: f64) {
    let E = 3. / 64. * M_PI * M * M / R;
    for _i in 0.._N {
        let mut star = reb_particle::default();
        let _r = (reb_random_uniform(Some(r), 0., 1.).powf(-2. / 3.) - 1.).powf(-1. / 2.);
        let x2 = reb_random_uniform(Some(r), 0., 1.);
        let x3 = reb_random_uniform(Some(r), 0., 2. * M_PI);
        star.z = (1. - 2. * x2) * _r;
        star.x = (_r * _r - star.z * star.z).sqrt() * x3.cos();
        star.y = (_r * _r - star.z * star.z).sqrt() * x3.sin();
        let mut x5;
        let mut g;
        let mut q;
        loop {
            x5 = reb_random_uniform(Some(r), 0., 1.);
            q = reb_random_uniform(Some(r), 0., 1.);
            g = q * q * (1. - q * q).powf(7. / 2.);
            if 0.1 * x5 <= g {
                break;
            }
        }
        let ve = 2.0_f64.powf(1. / 2.) * (1. + _r * _r).powf(-1. / 4.);
        let v = q * ve;
        let x6 = reb_random_uniform(Some(r), 0., 1.);
        let x7 = reb_random_uniform(Some(r), 0., 2. * M_PI);
        star.vz = (1. - 2. * x6) * v;
        star.vx = (v * v - star.vz * star.vz).sqrt() * x7.cos();
        star.vy = (v * v - star.vz * star.vz).sqrt() * x7.sin();

        star.x *= 3. * M_PI / 64. * M * M / E;
        star.y *= 3. * M_PI / 64. * M * M / E;
        star.z *= 3. * M_PI / 64. * M * M / E;

        star.vx *= (E * 64. / 3. / M_PI / M).sqrt();
        star.vy *= (E * 64. / 3. / M_PI / M).sqrt();
        star.vz *= (E * 64. / 3. / M_PI / M).sqrt();

        star.m = M / (_N as f64);

        crate::particle::reb_simulation_add(r, star);
    }
}

/// tools.c `reb_mod2pi`.
pub fn reb_mod2pi(f: f64) -> f64 {
    let pi2 = 2. * M_PI;
    (pi2 + f % pi2) % pi2
}

/// tools.c `reb_M_to_E` (Kepler's equation, Danby & Burkardt guess).
pub fn reb_M_to_E(e: f64, M: f64) -> f64 {
    let mut E;
    let mut F;
    let mut M = M;
    if e < 1. {
        M = reb_mod2pi(M);
        let mut sigma = 1.;
        if M > M_PI {
            sigma = -1.;
        }
        E = M + sigma * 0.71 * e;

        F = E - e * E.sin() - M;
        for _i in 0..100 {
            E = E - F / (1. - e * E.cos());
            F = E - e * E.sin() - M;
            if F.abs() < 1.0e-15 {
                break;
            }
        }
        E = reb_mod2pi(E);
    } else {
        E = M / M.abs() * (2. * M.abs() / e + 1.8).ln();

        F = E - e * E.sinh() + M;
        for _i in 0..100 {
            E = E - F / (1.0 - e * E.cosh());
            F = E - e * E.sinh() + M;
            if F.abs() < 1.0e-15 {
                break;
            }
        }
    }
    E
}

/// tools.c `reb_E_to_f`.
pub fn reb_E_to_f(e: f64, E: f64) -> f64 {
    if e > 1. {
        reb_mod2pi(2. * (((1. + e) / (e - 1.)).sqrt() * (0.5 * E).tanh()).atan())
    } else {
        reb_mod2pi(2. * (((1. + e) / (1. - e)).sqrt() * (0.5 * E).tan()).atan())
    }
}

/// tools.c `reb_M_to_f`.
pub fn reb_M_to_f(e: f64, M: f64) -> f64 {
    let E = reb_M_to_E(e, M);
    reb_E_to_f(e, E)
}

const TINY: f64 = 1.0e-308;
const MIN_INC: f64 = 1.0e-8;
const MIN_ECC: f64 = 1.0e-8;

/// tools.c `reb_particle_nan`.
pub fn reb_particle_nan() -> reb_particle {
    reb_particle {
        x: f64::NAN,
        y: f64::NAN,
        z: f64::NAN,
        vx: f64::NAN,
        vy: f64::NAN,
        vz: f64::NAN,
        ax: f64::NAN,
        ay: f64::NAN,
        az: f64::NAN,
        m: f64::NAN,
        r: f64::NAN,
        name: None,
    }
}

/// tools.c `reb_particle_from_orbit_err` (Murray & Dermott Eq 2.122 /
/// 2.36).
pub fn reb_particle_from_orbit_err(
    G: f64,
    primary: reb_particle,
    m: f64,
    a: f64,
    e: f64,
    inc: f64,
    Omega: f64,
    omega: f64,
    f: f64,
    err: &mut i32,
) -> reb_particle {
    if e == 1. {
        *err = 1;
        return reb_particle_nan();
    }
    if e < 0. {
        *err = 2;
        return reb_particle_nan();
    }
    if e > 1. {
        if a > 0. {
            *err = 3;
            return reb_particle_nan();
        }
    } else if a < 0. {
        *err = 4;
        return reb_particle_nan();
    }
    if e * f.cos() < -1. {
        *err = 5;
        return reb_particle_nan();
    }
    if primary.m < TINY {
        *err = 6;
        return reb_particle_nan();
    }

    let mut p = reb_particle::default();
    p.m = m;
    let r = a * (1. - e * e) / (1. + e * f.cos());
    let v0 = (G * (m + primary.m) / a / (1. - e * e)).sqrt();

    let cO = Omega.cos();
    let sO = Omega.sin();
    let co = omega.cos();
    let so = omega.sin();
    let cf = f.cos();
    let sf = f.sin();
    let ci = inc.cos();
    let si = inc.sin();

    p.x = primary.x + r * (cO * (co * cf - so * sf) - sO * (so * cf + co * sf) * ci);
    p.y = primary.y + r * (sO * (co * cf - so * sf) + cO * (so * cf + co * sf) * ci);
    p.z = primary.z + r * (so * cf + co * sf) * si;

    p.vx = primary.vx
        + v0 * ((e + cf) * (-ci * co * sO - cO * so) - sf * (co * cO - ci * so * sO));
    p.vy = primary.vy
        + v0 * ((e + cf) * (ci * co * cO - sO * so) - sf * (co * sO + ci * so * cO));
    p.vz = primary.vz + v0 * ((e + cf) * co * si - sf * si * so);

    p.ax = 0.;
    p.ay = 0.;
    p.az = 0.;

    p
}

/// tools.c `reb_particle_from_orbit`.
pub fn reb_particle_from_orbit(
    G: f64,
    primary: reb_particle,
    m: f64,
    a: f64,
    e: f64,
    inc: f64,
    Omega: f64,
    omega: f64,
    f: f64,
) -> reb_particle {
    let mut err = 0;
    reb_particle_from_orbit_err(G, primary, m, a, e, inc, Omega, omega, f, &mut err)
}

fn reb_orbit_nan() -> reb_orbit {
    let mut o = reb_orbit::default();
    o.d = f64::NAN;
    o.v = f64::NAN;
    o.h = f64::NAN;
    o.P = f64::NAN;
    o.n = f64::NAN;
    o.a = f64::NAN;
    o.e = f64::NAN;
    o.inc = f64::NAN;
    o.Omega = f64::NAN;
    o.omega = f64::NAN;
    o.pomega = f64::NAN;
    o.f = f64::NAN;
    o.M = f64::NAN;
    o.l = f64::NAN;
    o.theta = f64::NAN;
    o.T = f64::NAN;
    o.rhill = f64::NAN;
    o
}

/// tools.c `acos2` — quadrant-correct acos(num/denom).
fn acos2(num: f64, denom: f64, disambiguator: f64) -> f64 {
    let cosine = num / denom;
    if cosine > -1. && cosine < 1. {
        let mut val = cosine.acos();
        if disambiguator < 0. {
            val = -val;
        }
        val
    } else if cosine <= -1. {
        M_PI
    } else {
        0.
    }
}

/// tools.c `reb_orbit_from_particle_err`. The C reads the time of the
/// particle's simulation through the `sim` back-pointer; here the time
/// is passed explicitly as `t0` (pass `0.0` for a free particle, which
/// is the C behavior when `p.sim == NULL`).
pub fn reb_orbit_from_particle_err_t(
    G: f64,
    p: reb_particle,
    primary: reb_particle,
    t0: f64,
    err: &mut i32,
) -> reb_orbit {
    let mut o = reb_orbit::default();
    if primary.m <= TINY {
        *err = 1;
        return reb_orbit_nan();
    }
    let mu = G * (p.m + primary.m);
    let dx = p.x - primary.x;
    let dy = p.y - primary.y;
    let dz = p.z - primary.z;
    let dvx = p.vx - primary.vx;
    let dvy = p.vy - primary.vy;
    let dvz = p.vz - primary.vz;
    o.d = (dx * dx + dy * dy + dz * dz).sqrt();

    let vsquared = dvx * dvx + dvy * dvy + dvz * dvz;
    o.v = vsquared.sqrt();
    let vcircsquared = mu / o.d;
    o.a = -mu / (vsquared - 2. * vcircsquared);

    o.rhill = o.a * (p.m / (3. * primary.m)).cbrt();

    let hx = dy * dvz - dz * dvy;
    let hy = dz * dvx - dx * dvz;
    let hz = dx * dvy - dy * dvx;
    o.h = (hx * hx + hy * hy + hz * hz).sqrt();
    o.hvec = reb_vec3d { x: hx, y: hy, z: hz };

    let vdiffsquared = vsquared - vcircsquared;
    if o.d <= TINY {
        *err = 2;
        return reb_orbit_nan();
    }
    let vr = (dx * dvx + dy * dvy + dz * dvz) / o.d;
    let rvr = o.d * vr;
    let muinv = 1. / mu;

    let ex = muinv * (vdiffsquared * dx - rvr * dvx);
    let ey = muinv * (vdiffsquared * dy - rvr * dvy);
    let ez = muinv * (vdiffsquared * dz - rvr * dvz);
    o.e = (ex * ex + ey * ey + ez * ez).sqrt();
    o.evec = reb_vec3d { x: ex, y: ey, z: ez };
    o.n = o.a / o.a.abs() * (mu / (o.a * o.a * o.a)).abs().sqrt();
    o.P = 2. * M_PI / o.n;

    o.inc = acos2(hz, o.h, 1.);

    let nx = -hy;
    let ny = hx;
    let n = (nx * nx + ny * ny).sqrt();

    o.Omega = acos2(nx, n, ny);

    let ea;
    if o.e < 1. {
        ea = acos2(1. - o.d / o.a, o.e, vr);
        o.M = ea - o.e * ea.sin();
    } else {
        let mut ea_h = ((1. - o.d / o.a) / o.e).acosh();
        if vr < 0. {
            ea_h = -ea_h;
        }
        o.M = o.e * ea_h.sinh() - ea_h;
    }

    if o.inc < MIN_INC || o.inc > M_PI - MIN_INC {
        // nearly planar
        o.theta = acos2(dx, o.d, dy);
        o.pomega = acos2(ex, o.e, ey);

        if o.inc < M_PI / 2. {
            o.omega = o.pomega - o.Omega;
            o.f = o.theta - o.pomega;
            if o.e > MIN_ECC {
                o.l = o.pomega + o.M;
            } else {
                o.l = o.theta - 2. * o.e * o.f.sin();
            }
        } else {
            o.omega = o.Omega - o.pomega;
            o.f = o.pomega - o.theta;
            if o.e > MIN_ECC {
                o.l = o.pomega - o.M;
            } else {
                o.l = o.theta + 2. * o.e * o.f.sin();
            }
        }
    } else {
        // non-planar
        let wpf = acos2(nx * dx + ny * dy, n * o.d, dz);
        o.omega = acos2(nx * ex + ny * ey, n * o.e, ez);
        if o.inc < M_PI / 2. {
            o.pomega = o.Omega + o.omega;
            o.f = wpf - o.omega;
            o.theta = o.Omega + wpf;
            if o.e > MIN_ECC {
                o.l = o.pomega + o.M;
            } else {
                o.l = o.theta - 2. * o.e * o.f.sin();
            }
        } else {
            o.pomega = o.Omega - o.omega;
            o.f = wpf - o.omega;
            o.theta = o.Omega - wpf;
            if o.e > MIN_ECC {
                o.l = o.pomega - o.M;
            } else {
                o.l = o.theta + 2. * o.e * o.f.sin();
            }
        }
    }

    o.T = t0 - o.M / o.n.abs();

    o.Omega = reb_mod2pi(o.Omega);
    o.pomega = reb_mod2pi(o.pomega);
    o.f = reb_mod2pi(o.f);
    o.l = reb_mod2pi(o.l);
    o.M = reb_mod2pi(o.M);
    o.theta = reb_mod2pi(o.theta);
    o.omega = reb_mod2pi(o.omega);

    // Pal (2009) coordinates
    let fac = (2. / (1. + hz / o.h)).sqrt() / o.h;
    o.pal_ix = -fac * hy;
    o.pal_iy = fac * hx;
    o.pal_k = o.h / mu * (dvy - dvz / (o.h + hz) * hy) - 1. / o.d * (dx - dz / (o.h + hz) * hx);
    o.pal_h = o.h / mu * (-dvx + dvz / (o.h + hz) * hx) - 1. / o.d * (dy - dz / (o.h + hz) * hy);
    o
}

/// tools.c `reb_orbit_from_particle` (t0 = 0, the free-particle case).
pub fn reb_orbit_from_particle(G: f64, p: reb_particle, primary: reb_particle) -> reb_orbit {
    let mut err = 0;
    reb_orbit_from_particle_err_t(G, p, primary, 0.0, &mut err)
}

/// tools.c `reb_tools_solve_kepler_pal`.
pub fn reb_tools_solve_kepler_pal(h: f64, k: f64, lambda: f64, p: &mut f64, q: &mut f64) {
    let e2 = h * h + k * k;
    if e2 < 0.3 * 0.3 {
        let mut pn: f64 = 0.;
        let mut qn: f64 = 0.;

        let mut n = 0;
        loop {
            let f0 = qn * pn.cos() + pn * pn.sin() - (k * lambda.cos() + h * lambda.sin());
            let f1 = -qn * pn.sin() + pn * pn.cos() - (k * lambda.sin() - h * lambda.cos());

            let fac = 1. / (qn - 1.);
            let fd00 = fac * (qn * pn.cos() - pn.cos() + pn * pn.sin());
            let fd01 = fac * (pn * pn.cos() - qn * pn.sin() + pn.sin());
            let fd10 = fac * (-pn.sin());
            let fd11 = fac * (-pn.cos());

            qn -= fd00 * f0 + fd10 * f1;
            pn -= fd01 * f0 + fd11 * f1;
            let f = (f0 * f0 + f1 * f1).sqrt();
            let cont = n < 50 && f > 1e-15;
            n += 1;
            if !cont {
                break;
            }
        }
        *p = pn;
        *q = qn;
    } else {
        let pomega = h.atan2(k);
        let M = lambda - pomega;
        let e = e2.sqrt();
        let E = reb_M_to_E(e, M);
        *p = e * E.sin();
        *q = e * E.cos();
    }
}

/// tools.c `reb_tools_particle_to_pal`.
pub fn reb_tools_particle_to_pal(
    G: f64,
    p: reb_particle,
    primary: reb_particle,
    a: &mut f64,
    lambda: &mut f64,
    k: &mut f64,
    h: &mut f64,
    ix: &mut f64,
    iy: &mut f64,
) {
    let x = p.x - primary.x;
    let y = p.y - primary.y;
    let z = p.z - primary.z;
    let vx = p.vx - primary.vx;
    let vy = p.vy - primary.vy;
    let vz = p.vz - primary.vz;
    let mu = G * (p.m + primary.m);
    let r2 = x * x + y * y + z * z;
    let r = r2.sqrt();
    let cx = y * vz - z * vy;
    let cy = z * vx - x * vz;
    let cz = x * vy - y * vx;
    let c2 = cx * cx + cy * cy + cz * cz;
    let c = c2.sqrt();
    let chat = x * vx + y * vy + z * vz;

    let fac = (2. / (1. + cz / c)).sqrt() / c;
    *ix = -fac * cy;
    *iy = fac * cx;
    *k = c / mu * (vy - vz / (c + cz) * cy) - 1. / r * (x - z / (c + cz) * cx);
    *h = c / mu * (-vx + vz / (c + cz) * cx) - 1. / r * (y - z / (c + cz) * cy);
    let e2 = (*k) * (*k) + (*h) * (*h);
    *a = c2 / (mu * (1. - e2));
    let l = 1. - (1. - e2).sqrt();
    *lambda = (-r * vx + r * vz * cx / (c + cz) - (*k) * chat / (2. - l))
        .atan2(r * vy - r * vz * cy / (c + cz) + (*h) * chat / (2. - l))
        - chat / c * (1. - l);
}

/// tools.c `reb_particle_from_pal`.
pub fn reb_particle_from_pal(
    G: f64,
    primary: reb_particle,
    m: f64,
    a: f64,
    lambda: f64,
    k: f64,
    h: f64,
    ix: f64,
    iy: f64,
) -> reb_particle {
    let mut np = reb_particle::default();
    np.m = m;

    let mut p = 0.;
    let mut q = 0.;
    reb_tools_solve_kepler_pal(h, k, lambda, &mut p, &mut q);

    let slp = (lambda + p).sin();
    let clp = (lambda + p).cos();

    let l = 1. - (1. - h * h - k * k).sqrt();
    let xi = a * (clp + p / (2. - l) * h - k);
    let eta = a * (slp - p / (2. - l) * k - h);

    let iz = (4. - ix * ix - iy * iy).abs().sqrt();
    let W = eta * ix - xi * iy;

    np.x = primary.x + xi + 0.5 * iy * W;
    np.y = primary.y + eta - 0.5 * ix * W;
    np.z = primary.z + 0.5 * iz * W;

    let an = (G * (m + primary.m) / a).sqrt();
    let dxi = an / (1. - q) * (-slp + q / (2. - l) * h);
    let deta = an / (1. - q) * (clp - q / (2. - l) * k);
    let dW = deta * ix - dxi * iy;

    np.vx = primary.vx + dxi + 0.5 * iy * dW;
    np.vy = primary.vy + deta - 0.5 * ix * dW;
    np.vz = primary.vz + 0.5 * iz * dW;

    np
}

/// tools.c `reb_simulation_rescale_var`.
pub fn reb_simulation_rescale_var(r: &mut reb_simulation) {
    if r.var_config.is_empty() {
        return;
    }
    for v in 0..r.var_config.len() {
        let vc = r.var_config[v];
        if vc.lrescale < 0. {
            continue;
        }
        let N = if vc.testparticle < 0 { r.N } else { 1 };
        let mut scale: f64 = 0.;
        for i in 0..N {
            let p = r.particles_var[vc.index + i];
            scale = p.x.abs().max(scale);
            scale = p.y.abs().max(scale);
            scale = p.z.abs().max(scale);
            scale = p.vx.abs().max(scale);
            scale = p.vy.abs().max(scale);
            scale = p.vz.abs().max(scale);
        }
        if scale > 1e100 {
            if vc.order == 1 {
                for w in 0..r.var_config.len() {
                    let wc = r.var_config[w];
                    if wc.order == 2
                        && (wc.index_1st_order_a == vc.index || wc.index_1st_order_b == vc.index)
                        && (r.messages_var_rescale_warning & 4) == 0
                    {
                        r.messages_var_rescale_warning |= 4;
                        reb_simulation_warning(r, "Rescaling a set of variational equations of order 1 which are being used by a set of variational equations of order 2. Order 2 equations will no longer be valid.");
                    }
                }
            } else {
                if (r.messages_var_rescale_warning & 2) == 0 {
                    r.messages_var_rescale_warning |= 2;
                    reb_simulation_warning(r, "Variational particles which are part of a second order variational equation have now large coordinates which might exceed range of floating point number range. REBOUND cannot rescale a second order variational equation as it is non-linear.");
                }
                return;
            }
            if r.is_synchronized == 0 {
                if (r.messages_var_rescale_warning & 1) == 0 {
                    r.messages_var_rescale_warning |= 1;
                    reb_simulation_warning(r, "Variational particles have large coordinates which might exceed range of floating point numbers. Rescaling failed because integrator was not synchronized. Turn on safe_mode or manually synchronize. Then rescale.");
                }
                return;
            }
            r.var_config[v].lrescale += scale.ln();
            for i in 0..N {
                r.particles_var[vc.index + i].x /= scale;
                r.particles_var[vc.index + i].y /= scale;
                r.particles_var[vc.index + i].z /= scale;
                r.particles_var[vc.index + i].vx /= scale;
                r.particles_var[vc.index + i].vy /= scale;
                r.particles_var[vc.index + i].vz /= scale;
            }
            r.did_modify_particles = 1;
        }
    }
}

fn reb_simulation_add_var_particle_local(r: &mut reb_simulation, N_var_add: usize) {
    for _ in 0..N_var_add {
        r.particles_var.push(reb_particle::default());
    }
    r.N_var += N_var_add;
}

/// tools.c `reb_simulation_add_variation_1st_order`.
pub fn reb_simulation_add_variation_1st_order(r: &mut reb_simulation, testparticle: i32) -> usize {
    let index = r.N_var;
    r.var_config.push(reb_variational_configuration {
        order: 1,
        index,
        lrescale: 0.,
        testparticle,
        index_1st_order_a: 0,
        index_1st_order_b: 0,
    });
    if testparticle >= 0 {
        reb_simulation_add_var_particle_local(r, 1);
    } else {
        reb_simulation_add_var_particle_local(r, r.N);
    }
    index
}

/// tools.c `reb_simulation_add_variation_2nd_order`.
pub fn reb_simulation_add_variation_2nd_order(
    r: &mut reb_simulation,
    testparticle: i32,
    index_1st_order_a: usize,
    index_1st_order_b: usize,
) -> usize {
    let index = r.N_var;
    r.var_config.push(reb_variational_configuration {
        order: 2,
        index,
        lrescale: 0.,
        testparticle,
        index_1st_order_a,
        index_1st_order_b,
    });
    if testparticle >= 0 {
        reb_simulation_add_var_particle_local(r, 1);
    } else {
        reb_simulation_add_var_particle_local(r, r.N);
    }
    index
}

/// tools.c `reb_simulation_init_megno_seed`.
pub fn reb_simulation_init_megno_seed(r: &mut reb_simulation, seed: u32) {
    r.rand_seed = seed;
    reb_simulation_init_megno(r);
}

/// tools.c `reb_simulation_init_megno`.
pub fn reb_simulation_init_megno(r: &mut reb_simulation) {
    r.megno_Ys = 0.;
    r.megno_Yss = 0.;
    r.megno_cov_Yt = 0.;
    r.megno_var_t = 0.;
    r.megno_n = 0;
    r.megno_mean_Y = 0.;
    r.megno_initial_t = r.t;
    r.megno_mean_t = 0.;
    reb_simulation_add_variation_1st_order(r, -1);
    r.calculate_megno = 1;
    for i in 0..r.N {
        r.particles_var[i].m = 0.;
        r.particles_var[i].x = reb_random_normal(Some(r), 1.);
        r.particles_var[i].y = reb_random_normal(Some(r), 1.);
        r.particles_var[i].z = reb_random_normal(Some(r), 1.);
        r.particles_var[i].vx = reb_random_normal(Some(r), 1.);
        r.particles_var[i].vy = reb_random_normal(Some(r), 1.);
        r.particles_var[i].vz = reb_random_normal(Some(r), 1.);
        let p = r.particles_var[i];
        let deltad = 1.
            / (p.x * p.x + p.y * p.y + p.z * p.z + p.vx * p.vx + p.vy * p.vy + p.vz * p.vz)
                .sqrt();
        r.particles_var[i].x *= deltad;
        r.particles_var[i].y *= deltad;
        r.particles_var[i].z *= deltad;
        r.particles_var[i].vx *= deltad;
        r.particles_var[i].vy *= deltad;
        r.particles_var[i].vz *= deltad;
    }
}

/// tools.c `reb_simulation_megno`.
pub fn reb_simulation_megno(r: &reb_simulation) -> f64 {
    if r.t == r.megno_initial_t {
        return 0.;
    }
    r.megno_Yss / (r.t - r.megno_initial_t)
}

/// tools.c `reb_simulation_lyapunov` (Cincotta & Simo 2000, Eq 24).
pub fn reb_simulation_lyapunov(r: &reb_simulation) -> f64 {
    if r.megno_var_t == 0.0 {
        return 0.;
    }
    r.megno_cov_Yt / r.megno_var_t
}

/// tools.c `reb_tools_megno_deltad_delta`.
pub fn reb_tools_megno_deltad_delta(r: &reb_simulation) -> f64 {
    let mut deltad = 0.;
    let mut delta2 = 0.;
    for i in 0..r.N {
        let p = r.particles_var[i];
        deltad += p.vx * p.x;
        deltad += p.vy * p.y;
        deltad += p.vz * p.z;
        deltad += p.ax * p.vx;
        deltad += p.ay * p.vy;
        deltad += p.az * p.vz;
        delta2 += p.x * p.x;
        delta2 += p.y * p.y;
        delta2 += p.z * p.z;
        delta2 += p.vx * p.vx;
        delta2 += p.vy * p.vy;
        delta2 += p.vz * p.vz;
    }
    deltad / delta2
}

/// tools.c `reb_tools_megno_update`.
pub fn reb_tools_megno_update(r: &mut reb_simulation, dY: f64, dt_done: f64) {
    r.megno_Ys += dY;
    let Y = r.megno_Ys / (r.t - r.megno_initial_t);
    r.megno_Yss += Y * dt_done;
    r.megno_n += 1;
    let _d_t = r.t - r.megno_initial_t - r.megno_mean_t;
    r.megno_mean_t += _d_t / (r.megno_n as f64);
    let _d_Y = reb_simulation_megno(r) - r.megno_mean_Y;
    r.megno_mean_Y += _d_Y / (r.megno_n as f64);
    r.megno_cov_Yt += ((r.megno_n as f64) - 1.) / (r.megno_n as f64)
        * (r.t - r.megno_initial_t - r.megno_mean_t)
        * (reb_simulation_megno(r) - r.megno_mean_Y);
    r.megno_var_t += ((r.megno_n as f64) - 1.) / (r.megno_n as f64)
        * (r.t - r.megno_initial_t - r.megno_mean_t)
        * (r.t - r.megno_initial_t - r.megno_mean_t);
}

/// tools.c `reb_simulation_imul`.
pub fn reb_simulation_imul(r: &mut reb_simulation, scalar_pos: f64, scalar_vel: f64) {
    for i in 0..r.N {
        r.particles[i].x *= scalar_pos;
        r.particles[i].y *= scalar_pos;
        r.particles[i].z *= scalar_pos;
        r.particles[i].vx *= scalar_vel;
        r.particles[i].vy *= scalar_vel;
        r.particles[i].vz *= scalar_vel;
    }
}

/// tools.c `reb_simulation_iadd`.
pub fn reb_simulation_iadd(r: &mut reb_simulation, r2: &reb_simulation) -> i32 {
    if r.N != r2.N {
        return -1;
    }
    for i in 0..r.N {
        r.particles[i].x += r2.particles[i].x;
        r.particles[i].y += r2.particles[i].y;
        r.particles[i].z += r2.particles[i].z;
        r.particles[i].vx += r2.particles[i].vx;
        r.particles[i].vy += r2.particles[i].vy;
        r.particles[i].vz += r2.particles[i].vz;
    }
    0
}

/// tools.c `reb_simulation_isub`.
pub fn reb_simulation_isub(r: &mut reb_simulation, r2: &reb_simulation) -> i32 {
    if r.N != r2.N {
        return -1;
    }
    for i in 0..r.N {
        r.particles[i].x -= r2.particles[i].x;
        r.particles[i].y -= r2.particles[i].y;
        r.particles[i].z -= r2.particles[i].z;
        r.particles[i].vx -= r2.particles[i].vx;
        r.particles[i].vy -= r2.particles[i].vy;
        r.particles[i].vz -= r2.particles[i].vz;
    }
    0
}

/// tools.c `reb_tools_spherical_to_xyz`.
pub fn reb_tools_spherical_to_xyz(magnitude: f64, theta: f64, phi: f64) -> reb_vec3d {
    reb_vec3d {
        x: magnitude * theta.sin() * phi.cos(),
        y: magnitude * theta.sin() * phi.sin(),
        z: magnitude * theta.cos(),
    }
}

/// tools.c `reb_tools_xyz_to_spherical`.
pub fn reb_tools_xyz_to_spherical(xyz: reb_vec3d, magnitude: &mut f64, theta: &mut f64, phi: &mut f64) {
    *magnitude = (xyz.x * xyz.x + xyz.y * xyz.y + xyz.z * xyz.z).sqrt();
    *theta = acos2(xyz.z, *magnitude, 1.);
    *phi = xyz.y.atan2(xyz.x);
}

// ---- message helpers (simulation.c / rebound.c) --------------------------

/// simulation.c `reb_simulation_warning`.
pub fn reb_simulation_warning(r: &mut reb_simulation, msg: &str) {
    reb_message(r, REB_MESSAGE_TYPE::WARNING, msg);
}

/// simulation.c `reb_simulation_error`.
pub fn reb_simulation_error(r: &mut reb_simulation, msg: &str) {
    reb_message(r, REB_MESSAGE_TYPE::ERROR, msg);
}

/// simulation.c `reb_simulation_info`.
pub fn reb_simulation_info(r: &mut reb_simulation, msg: &str) {
    reb_message(r, REB_MESSAGE_TYPE::INFO, msg);
}

/// rebound.c `reb_message` (print-or-store; the ANSI color escapes of
/// the POSIX build are absent on Windows, exactly like the C).
pub fn reb_message(r: &mut reb_simulation, msg_type: REB_MESSAGE_TYPE, msg: &str) {
    if r.save_messages == 0 {
        eprintln!();
        match msg_type {
            REB_MESSAGE_TYPE::INFO => eprint!("REBOUND Message!"),
            REB_MESSAGE_TYPE::WARNING => eprint!("Warning!"),
            REB_MESSAGE_TYPE::ERROR => eprint!("Error!"),
        }
        eprintln!(" {}", msg);
    } else {
        if r.messages.len() == reb_messages_max_N {
            r.messages.remove(0);
        }
        r.messages.push((msg_type, msg.to_string()));
    }
}

/// rebound.c `reb_exit`.
pub fn reb_exit(msg: &str) -> ! {
    eprintln!();
    eprintln!("Error! {}", msg);
    std::process::exit(1);
}
