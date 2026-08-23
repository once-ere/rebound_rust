# rebound_rust (`rebound_rs`)

A pure-Rust translation of **[REBOUND](https://github.com/hannorein/rebound)** 5.1.1,
the open-source multi-purpose N-body code for collisional dynamics by
**Hanno Rein** and collaborators.

- Upstream source translated: https://github.com/hannorein/rebound, commit
  [`dad5f978`](https://github.com/hannorein/rebound/commit/dad5f97806ecbb408dcaff728851c64e67f9f6eb)
  ("Patch (#931)", version 5.1.1).
- Zero `unsafe`, zero external dependencies (std only), zero warnings.
- C function, struct and constant names are preserved exactly
  (`reb_simulation_create`, `reb_particle`, `reb_simulation_integrate`, ...).
- **Bit-for-bit verified** against the MSVC-compiled C reference build on
  Windows 11: 63 integrator configurations, a 1482-particle shearing-sheet run
  with ~10⁵ collisions (identical SHA-256 state dumps), all 65 orbital
  derivative functions, the frequency analysis in all three modes, and
  Simulationarchive round trips in both directions (archives written by this
  crate load bit-exactly in C-REBOUND and vice versa).
- Full provenance — how the C reference was built, how every file was
  translated, every command used for building and verification, and complete
  usage instructions — is in [`rebound_rust.md`](rebound_rust.md)
  (also typeset as [`rebound_rust.pdf`](rebound_rust.pdf)). The
  shearing-sheet acceptance test is documented in
  [`shearing_sheet_port_test.md`](shearing_sheet_port_test.md).
- Every example in [`examples/`](examples/) has a companion Jupyter notebook
  in [`notebooks/`](notebooks/).

## Quick start

```toml
[dependencies]
rebound_rs = { path = "path/to/rebound_rust" }
```

```rust
use rebound_rs::*;

fn main() {
    let mut sim = reb_simulation_create();
    let r = &mut sim;
    reb_simulation_set_integrator(r, "whfast");
    r.G = 1.0;
    r.dt = 0.01;

    let mut star = reb_particle::default();
    star.m = 1.0;
    reb_simulation_add(r, star);

    reb_simulation_add_fmt(r, "m a e", &[
        reb_fmt_arg::d(1e-3), reb_fmt_arg::d(1.0), reb_fmt_arg::d(0.1),
    ]);

    reb_simulation_move_to_com(r);
    reb_simulation_integrate(r, 100.0);
    println!("t = {}  E = {}", r.t, reb_simulation_energy(r));
}
```

## Attribution and how to cite

**All scientific credit for the algorithms and the original implementation
belongs to the REBOUND authors.** REBOUND is (c) Hanno Rein, Shangfei Liu,
Dan Tamayo, David S. Spiegel, Daniel Tamayo, Tiger Lu, Pejvak Javaheri,
Rishit Dagli, Dave O'Hallaron, Ernst Hairer, and the REBOUND contributors.
This crate is a derivative work: a line-for-line translation of their C code,
distributed under the same license (GPL-3.0-or-later). Every Rust module
header cites the C file it translates and its copyright holders.

From the REBOUND project:

> If you use this code or parts of this code for results presented in a
> scientific publication, we would greatly appreciate a citation. The
> simplest way to find the citations relevant to the specific setup of your
> REBOUND simulation is:
>
> ```python
> sim = rebound.Simulation()
> # -your setup-
> sim.cite()
> ```

The `sim.cite()` helper is part of the upstream Python package
([`pip install rebound`](https://rebound.readthedocs.io)); it prints the
exact citation list for the modules your simulation uses. This Rust port does
not re-implement it, so the table below maps this crate's modules to the
papers `sim.cite()` would point you to. **At minimum, please cite the main
REBOUND code paper (Rein & Liu 2012) for any use of this crate.**

| If you use... | Please cite |
|---|---|
| REBOUND at all (any module of this crate) | Rein & Liu 2012, *REBOUND: an open-source multi-purpose N-body code for collisional dynamics*, A&A 537, A128 — [ADS](https://ui.adsabs.harvard.edu/abs/2012A%26A...537A.128R) |
| SEI integrator / shearing sheet (`integrator_sei`, `boundary` shear) | Rein & Tremaine 2011, MNRAS 415, 3168 — [ADS](https://ui.adsabs.harvard.edu/abs/2011MNRAS.415.3168R) |
| IAS15 integrator (`integrator_ias15`) | Rein & Spiegel 2015, MNRAS 446, 1424 — [ADS](https://ui.adsabs.harvard.edu/abs/2015MNRAS.446.1424R) |
| IAS15 adaptive timestep (default `adaptive_mode`, PRS23) | Pham, Rein & Spiegel 2024, OJAp 7, 1 — [ADS](https://ui.adsabs.harvard.edu/abs/2024OJAp....7E...1P) |
| WHFast integrator (`integrator_whfast`) | Rein & Tamayo 2015, MNRAS 452, 376 — [ADS](https://ui.adsabs.harvard.edu/abs/2015MNRAS.452..376R); and Wisdom & Holman 1991, AJ 102, 1528 — [ADS](https://ui.adsabs.harvard.edu/abs/1991AJ....102.1528W) |
| WHFast kernels / high-order variants, SABA family (`integrator_whfast` kernels, `integrator_saba`) | Rein, Tamayo & Brown 2019, MNRAS 489, 4632 — [ADS](https://ui.adsabs.harvard.edu/abs/2019MNRAS.489.4632R) |
| WHFast512 (`integrator_whfast512`; stub on this platform) | Javaheri, Rein & Tamayo 2023, OJAp 6, 29 — [ADS](https://ui.adsabs.harvard.edu/abs/2023OJAp....6E..29J) |
| JANUS integrator (`integrator_janus`) | Rein & Tamayo 2018, MNRAS 473, 3351 — [ADS](https://ui.adsabs.harvard.edu/abs/2018MNRAS.473.3351R) |
| MERCURIUS integrator (`integrator_mercurius`) | Rein, Hernandez, Tamayo, Brown, Eckels, Holmes, Lau, Leblanc & Silburt 2019, MNRAS 485, 5490 — [ADS](https://ui.adsabs.harvard.edu/abs/2019MNRAS.485.5490R); and Chambers 1999, MNRAS 304, 793 — [ADS](https://ui.adsabs.harvard.edu/abs/1999MNRAS.304..793C) |
| EOS integrator (`integrator_eos`) | Rein 2020, MNRAS 492, 5413 — [ADS](https://ui.adsabs.harvard.edu/abs/2020MNRAS.492.5413R) |
| TRACE integrator (`integrator_trace`) | Lu, Hernandez & Rein 2024, MNRAS 533, 3708 — [ADS](https://ui.adsabs.harvard.edu/abs/2024MNRAS.533.3708L); and Hernandez & Dehnen 2023, MNRAS 522, 4639 — [ADS](https://ui.adsabs.harvard.edu/abs/2023MNRAS.522.4639H) |
| BS integrator / ODE framework (`integrator_bs`) | Rein & Liu 2012 (above); the implementation follows Hairer, Nørsett & Wanner 1993, *Solving Ordinary Differential Equations I* (Sect. II.9), via the [Hipparchus](https://hipparchus.org) Gragg–Bulirsch–Stoer implementation ((c) 2004 Ernst Hairer) |
| Simulationarchive (`simulationarchive`, `binarydata`) | Rein & Tamayo 2017, MNRAS 467, 2377 — [ADS](https://ui.adsabs.harvard.edu/abs/2017MNRAS.467.2377R) |
| Variational equations / MEGNO (`tools`, `derivatives`) | Rein & Tamayo 2016, MNRAS 459, 2275 — [ADS](https://ui.adsabs.harvard.edu/abs/2016MNRAS.459.2275R) |
| Frequency analysis (`frequency_analysis`) | Šidlichovský & Nesvorný 1996, CeMDA 65, 137 — [ADS](https://ui.adsabs.harvard.edu/abs/1996CeMDA..65..137S); Laskar 1988, A&A 198, 341 — [ADS](https://ui.adsabs.harvard.edu/abs/1988A%26A...198..341L); based on David Nesvorný's [FMFT code](https://www2.boulder.swri.edu/~davidn/fmft/fmft.html) |
| Opening-angle tree gravity (`tree`, `gravity`) | Rein & Liu 2012 (above); Barnes & Hut 1986, Nature 324, 446 — [ADS](https://ui.adsabs.harvard.edu/abs/1986Natur.324..446B) |

BibTeX entries for all of these are collected in the upstream repository:
https://github.com/hannorein/rebound#papers and
https://rebound.readthedocs.io/en/latest/citations/.

If you additionally wish to reference this Rust translation, cite the
upstream papers above first, and link this repository for the translation
itself.

## Verification summary

| Test | Result |
|---|---|
| 63 integrator configurations, 500 steps (three-body) | bit-identical to MSVC C build |
| Shearing sheet: 1482 particles, 400 steps, 102,533 collisions | byte-identical dumps, equal SHA-256 |
| All 65 `reb_particle_derivative_*` functions | 130/130 lines bit-identical |
| Frequency analysis (MFT, FMFT, FMFT2) | bit-identical |
| Simulationarchive C→Rust and Rust→C load-and-continue | bit-identical continuations |
| Webserver `/simulation` blob loaded by the C build | bit-identical state |

Known limitation: `pow` is the one libm function where Rust does not defer to
the UCRT (≤2 ulp difference on ~0.03% of inputs); all details in
[`rebound_rust.md`](rebound_rust.md) §9 and
[`shearing_sheet_port_test.md`](shearing_sheet_port_test.md).

## License

GPL-3.0-or-later, the same license as REBOUND. See [`LICENSE`](LICENSE).
REBOUND is free software; this translation is and remains free software.
