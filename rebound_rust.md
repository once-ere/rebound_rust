# rebound_rust — Provenance of the Pure-Rust Port of REBOUND 5.1.1

This document is the complete, self-contained provenance of the port of the
REBOUND N-body code from C to pure Rust on Windows 11: how the C reference was
obtained and built, how every C source file was translated, every command used
to build the Rust crate, how the port was verified bit-for-bit against the
MSVC-compiled C reference, and complete instructions for using the pure Rust
code. Every command below is given exactly and completely; nothing requires
consulting any other document.

- Crate: `rebound_rs` version 5.1.1, at
  `C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust`
- Upstream: https://github.com/hannorein/rebound, commit
  `dad5f97806ecbb408dcaff728851c64e67f9f6eb` ("Patch (#931)", version 5.1.1)
- License: GPL-3.0-or-later (the port is a derivative work of REBOUND,
  (c) Hanno Rein and collaborators; the LICENSE file is carried in the crate)
- Result: **every ported subsystem reproduces the MSVC C reference build
  bit-for-bit** — 63 integrator configurations, a 1482-particle shearing-sheet
  run with ~10^5 collisions (identical SHA-256 of the state dump), all 65
  orbital-derivative functions, the frequency analysis in all three modes, and
  cross-language Simulationarchive round trips in both directions.

---

## 1. Machine and toolchain

| Component | Value |
|---|---|
| OS | Windows 11 Pro for Workstations 10.0.26200, x86-64 (no WSL2 involved anywhere) |
| C compiler | MSVC `cl` 19.51.36256 for x64 (Visual Studio 2026 Build Tools) |
| MSVC environment | `"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat"` |
| make | GnuWin32 Make 3.81 (`winget install GnuWin32.Make`), installed at `C:\Program Files (x86)\GnuWin32\bin\make.exe` |
| vcpkg | `C:\Users\nsh\vcpkg\vcpkg.exe` |
| Rust | `rustc 1.91.1 (ed61e7d7e 2025-11-07)`, `cargo 1.91.1 (ea2d97820 2025-10-10)`, `x86_64-pc-windows-msvc` |

No GCC, no Clang, no WSL2 compiler was used at any point.

## 2. Step 1 — GLFW via vcpkg

REBOUND's native OpenGL visualization uses GLFW on POSIX systems. It was
installed for future use:

```
C:\Users\nsh\vcpkg\vcpkg.exe install glfw3:x64-windows
```

This produced glfw3 **3.5.1** with import libraries at
`C:\Users\nsh\vcpkg\installed\x64-windows\lib\glfw3dll.lib` and headers at
`C:\Users\nsh\vcpkg\installed\x64-windows\include\GLFW\`. Note, however, that
REBOUND's own build system **forces `OPENGL=0` on Windows**
(`src/Makefile.defs:21` prints "OpenGL not supported on Windows. Setting
OPENGL=0"), so the C reference build below does not link GLFW; the browser
based visualization (SERVER mode) is used instead. The vcpkg installation is
in place for any future work that compiles the POSIX OpenGL display path.

## 3. Step 2 — Cloning and building the C reference

```
cd C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0
git clone https://github.com/hannorein/rebound.git rebound\rebound
```

The clone is at commit `dad5f97806ecbb408dcaff728851c64e67f9f6eb`. The
shearing-sheet example (and with it `librebound.dll` / `librebound.lib`) was
built with GnuWin32 make driving MSVC `cl`. Two Windows-specific traps and
their solutions:

1. **PATH ordering**: `%PATH%` inside a `cmd /c '...'` string expands when the
   line is parsed, *before* `vcvars64.bat` runs — prepending GnuWin32 after
   vcvars would silently erase the vcvars additions. GnuWin32 must be
   prepended FIRST, then vcvars run:

```
cd C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound\rebound\examples\shearing_sheet
cmd /c 'set PATH=C:\Program Files (x86)\GnuWin32\bin;%PATH% && "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && make'
```

2. **Shell choice**: compiling C through Git-Bash's `cmd //c` can fail
   silently and leave a stale `.exe`; all C compilations were therefore done
   through PowerShell's `cmd /c`.

The build compiles every C file as, e.g.:

```
cl -c /DBUILDINGLIBREBOUND /D_GNU_SOURCE /D_CRT_SECURE_NO_WARNINGS /D_CRT_NONSTDC_NO_WARNINGS /Ox /fp:precise -DSERVER -DGITHASH=dad5f97806ecbb408dcaff728851c64e67f9f6eb /Fo:rebound.obj rebound.c
```

i.e. **`/Ox /fp:precise`, OPENGL off, SERVER on, no OpenMP, no MPI, no
AVX-512** (the `integrator_whfast512.s` assembly is only compiled with
GCC/Clang; under `cl` the file compiles to error stubs). This exact binary —
`librebound.lib`/`librebound.dll` and `rebound.exe` in the shearing_sheet
example directory — is the reference every Rust result is compared against.
An important detail for reproducibility: on Windows, REBOUND vendors glibc's
`rand_r` LCG directly in `rebound.c` (three rounds of
`seed*1103515245+12345`, `REB_RAND_MAX = 2147483647`), so random initial
conditions are identical across platforms — and identical in this port.

## 4. The Rust crate

```
rebound_rust\
├── Cargo.toml            rebound_rs 5.1.1, GPL-3.0-or-later, [workspace], zero dependencies
├── LICENSE               GPL v3
├── src\                  29 modules, 19,075 lines
├── examples\             9 runnable examples (each with a Jupyter notebook, see §10)
├── notebooks\            one notebook per example
├── porttest\             C reference harnesses + comparison artifacts
└── logs\                 build/run logs
```

Translation rules (identical to the sundials_rs porting discipline of the
surrounding rustSolveIt project):

- `#![forbid(unsafe_code)]`, `#![deny(warnings)]` — **zero `unsafe`, zero
  external dependencies (std only), zero warnings**;
- C function, struct and constant **names are preserved exactly**
  (`reb_simulation_create`, `reb_particle`, `reb_integrator_whfast_state`,
  ...) via `#![allow(non_snake_case, non_camel_case_types,
  non_upper_case_globals)]`;
- control flow, constants and **arithmetic order are preserved expression by
  expression** (floating-point addition is not associative; the C's exact
  evaluation order is the specification);
- missing/unportable symbols are reported in §6, never invented.

The `[workspace]` stanza in Cargo.toml is required: a stray, syntactically
broken `C:\Users\nsh\Developer\github\Cargo.toml` higher up the directory
tree would otherwise be picked up by cargo's upward workspace discovery and
break every build.

### 4.1 File-by-file accounting (all 31 C translation units)

| C file (lines) | Rust module | Status |
|---|---|---|
| rebound.c (~430 + data) | tools.rs, server.rs, lib.rs | Ported: messages, `reb_exit`, `reb_strcmp_ignore_whitespace`, `reb_check_fp_contract`, favicon PNG data, version/githash constants. N/A in Rust: `malloc`/aligned-alloc wrappers, SIGINT handler, custom-integrator registry (the integrator set is a Rust enum), `reb_avx512_available` (see whfast512), OpenMP thread setter. |
| simulation.c (874) | simulation.rs | Ported + verified (create/defaults, set_integrator, step, integrate, steps, synchronize, update_acceleration, exit checks, user-ODE post-step block, server/archive hooks). |
| particle.c | particle.rs | Ported + verified (add, remove, names/hash lookup, cmp, testparticle checks). |
| tools.c (1585) | tools.rs | Ported + verified (glibc-`rand_r`-exact RNG family, energy/angular momentum, com family incl. `reb_simulation_jacobi_com`, move_to_com/hel, plummer, orbit/particle conversions incl. Pal coordinates, `reb_simulation_add_fmt` + solar-system datasets, MEGNO/variational helpers, i-arithmetic operators). |
| boundary.c | boundary.rs | Ported + verified (incl. shear ghost boxes). |
| tree.c | tree.rs | Ported + verified (octree as index arena; `REB_TREECELL_NONE = usize::MAX` plays C's NULL). |
| gravity.c (926) | gravity.rs | Ported + verified (basic, compensated, tree, jacobi; variational + jerk terms). |
| collision.c (737) | collision.rs | Ported + verified (direct, line, tree, linetree searches; hardsphere/halt/merge resolvers; `rand_r` shuffle order preserved). |
| output.c | output.rs | Ported (timing with C-printf-compatible formatting, ascii, orbits, velocity dispersion, output checks). `reb_simulation_output_screenshot` is display-coupled and excluded (§6). |
| transformations.c (644) | transformations.rs | Ported + verified (Jacobi/DH/WHDS/barycentric, all directions, incl. variational). |
| rotations.c | rotations.rs | Ported + verified. |
| derivatives.c (2298) | derivatives.rs | Ported + verified: **all 65** `reb_particle_derivative_*` functions (the count in rebound.h is 65). |
| frequency_analysis.c (632) | frequency_analysis.rs | Ported + verified (Laskar MFT, Sidlichovsky–Nesvorny FMFT/FMFT2; NR-style FFT; golden-section maximization). |
| integrator_none.c | integrator_none.rs | Ported + verified. |
| integrator_sei.c | integrator_sei.rs | Ported + verified (via the shearing-sheet byte-identity run). |
| integrator_leapfrog.c | integrator_leapfrog.rs | Ported + verified (orders 2, 4, 6, 8). |
| integrator_ias15.c (1067) | integrator_ias15.rs | Ported + verified (adaptive PRS23 timestep, compensated sums, dp7 arrays, restart arrays `br`/`er`, map support). |
| integrator_whfast.c (1265) | integrator_whfast.rs | Ported + verified (Stumpff/Stiefel functions, Kepler solver with Newton + quartic + bisection paths, correctors 3/5/7/11/17 + corrector2, 4 kernels, 4 coordinate systems, variational equations). |
| integrator_saba.c | integrator_saba.rs | Ported + verified (all 13 tested type ids incl. CM/CL variants). |
| integrator_janus.c | integrator_janus.rs | Ported + verified (int64 grid; C's truncating double→int64 conversion is Rust's `as i64`; orders 2–10). |
| integrator_eos.c (670) | integrator_eos.rs | Ported + verified (all 9 phi splittings, processed methods with modified kicks). |
| integrator_mercurius.c (924) | integrator_mercurius.rs | Ported + verified (encounter prediction, IAS15 encounter sub-integration, dcrit criteria incl. `cbrt`, all 4 switching functions, add/remove-particle hooks). |
| integrator_bs.c (841) | integrator_bs.rs | Ported + verified (Gragg–Bulirsch–Stoer with the full `reb_ode` framework: `reb_ode_create/free`, modified midpoint, Richardson extrapolation, order/stepsize control). |
| integrator_trace.c (1262) | integrator_trace.rs | Ported + verified (reversible pre/post timestep checks with step rejection, K_ij matrix, BS/IAS15 encounter and pericenter prescriptions, hooks). |
| integrator_whfast512.c (652) | integrator_whfast512.rs | **Windows-stub parity**: under MSVC `cl` the C compiles only the `#else // Not 64 bit, Windows + cl` branch (step/synchronize raise "AVX512 is not supported on your platform." and set an error status). The Rust reproduces exactly that reference behavior, plus the state struct and its `create` defaults. The AVX-512 assembly core (`integrator_whfast512.s`) is not part of the Windows reference build. |
| binarydata.c (927) | binarydata.rs | Ported + verified. The C serializes via `offsetof` field-descriptor tables over raw struct memory; safe Rust reproduces the **same byte format** with explicit per-field serializers in the same field order (details §5.4). |
| simulationarchive.c (668) | simulationarchive.rs | Ported + verified (snapshot index construction with the blob-offset checksum, load-snapshot, append-diff save, auto interval/walltime/step heartbeat). |
| server.c (763) | server.rs | Ported + verified (HTTP endpoints `/`, `/simulation`, `/keyboard/<n>`, `/favicon.ico`, `/screenshot`; base64 decoder; rebound.html auto-download via curl; threading model adapted, §5.5). |
| fmemopen.c | — | N/A by construction: the C needs fmemopen to read archives from memory; Rust uses `std::io::Cursor`, which the simulationarchive module does. |
| display.c (1749), glad.c (1714), simplefont.h, khrplatform.h | — | **Excluded**: OpenGL display. The Windows C reference build itself compiles none of this (`OPENGL=0` forced); visualization is via the server + rebound.html, which IS ported. |
| communication_mpi.c | — | **Excluded**: MPI. Not part of the Windows reference build (and `MPI=1` does not compile under `cl`). |

## 5. Deviation classes (all mechanical, none arithmetic)

1. **Ownership instead of pointers.** `r->particles` (malloc) becomes
   `Vec<reb_particle>`; the octree's individually-malloc'd cells become an
   index arena rebuilt exactly as the C rebuilds cells; `p->name` (interned
   `char*`) becomes `Option<usize>` into `name_list`; particle `ap`/`sim`
   back-pointers are dropped (REBOUNDx is C-only; functions that used `sim`
   take the simulation explicitly, e.g. `reb_orbit_from_particle_err_t`
   carries the time as a parameter).
2. **Integrator state.** The C's `void* state` behind a vtable becomes
   `enum reb_integrator_state` with one variant per integrator. A step
   takes the state out of the enum (`std::mem::replace`) and puts it back —
   the C aliases `r` and `state` simultaneously; Rust makes that explicit.
   MERCURIUS/TRACE store the state back into `r.integrator` while their
   encounter sub-integrations run, so the custom gravity routines and the
   add/remove-particle hooks can reach it exactly where the C reads
   `r->integrator.state`.
3. **`r->map` aliasing.** The C sets `r->map = trace->encounter_map` (one
   array, two names). The Rust *moves* the Vec into `r.map` for the duration
   and moves it back — same array contents at every observable point.
4. **Binary format pointers.** `struct reb_particle` is serialized in its
   112-byte x86-64 memory layout. The C writes real heap pointers in the
   `name`/`ap`/`sim` slots; these are only ever compared for equality against
   pointers stored alongside `name_list`. The Rust writes 0 for `ap`/`sim`
   and a synthetic id for `name`, reproducing the protocol exactly — archives
   are interchangeable between the two implementations (verified, §7.5).
5. **Server threading.** The C server thread dereferences the simulation
   directly under a shared mutex. Safe Rust cannot alias `&mut
   reb_simulation` across threads; the same handshake is expressed with a
   shared snapshot/key-queue object serviced by the integrate loop at exactly
   the points where the C locks/unlocks its mutex. HTTP behavior is
   unchanged (verified, §7.6).
6. **Varargs.** `reb_simulation_add_fmt(r, fmt, ...)` takes its values as an
   ordered `&[reb_fmt_arg]` slice consumed token-by-token like `va_arg`.
7. **Sorting.** frequency_analysis' final amplitude sort uses Rust's stable
   sort where the C uses `qsort`; for distinct amplitudes (the generic case)
   the permutation is identical.
8. **Removed platform branches.** OpenMP `#pragma` branches (build has no
   OpenMP), `reb_sigint` polling (no signal handler), MPI branches. None of
   these executes in the C reference build.
9. **Uninitialized C memory.** e.g. `reb_orbit_nan` leaves some members
   uninitialized in C; Rust zero-initializes before setting the same members.

Nothing else deviates. In particular every floating-point expression, every
loop bound, every branch condition, and every RNG call sequence matches the C.

## 6. Symbols not carried (reported, not invented)

- `reb_simulation_output_screenshot` (output.c): requires the browser
  display round-trip; excluded with the display subsystem.
- `reb_integrator_register` / custom integrator registry (rebound.c): the
  Rust integrator set is a closed enum; registering external integrators at
  runtime is a C-API concept with no safe-Rust equivalent in this design.
- WHFast512 compute core (`integrator_whfast512.s`): AVX-512 assembly, not
  compiled by the MSVC reference build; the stub behavior is carried instead.
- MPI (`communication_mpi.c`) and OpenGL (`display.c`, `glad.c`) subsystems:
  excluded as above.

## 7. Verification — methodology, commands, results

The methodology throughout: run the identical experiment in the MSVC C
reference and in Rust, dump every result as raw IEEE-754 bit patterns
(`memcpy` to `unsigned long long` / `f64::to_bits`), and compare the dumps
byte for byte. A run passes only if **every bit of every value** matches.

The libm foundation that makes this possible (established with a dedicated
differential harness, `porttest/libm_diff.c` vs `examples/libm_diff.rs`,
200,000 samples per function): Rust on `x86_64-pc-windows-msvc` defers to the
same UCRT libm as `cl` for `sin`, `cos`, `tan`, `atan2`, `sqrt`, `fmod`,
`exp`, `log` — all bit-identical — while **`pow` is the single divergent
function** (Rust ships its own `pow`; 60/200,000 sampled inputs differ by ≤2
ulp). `cbrt` was additionally exercised bit-exactly through the MERCURIUS
dcrit criteria and the `add_fmt` period path. Consequence: any REBOUND code
path that calls `pow` at runtime can differ in the last bits (see §9); every
other path is exactly reproducible. Notably, all `powf` calls actually
executed in the BS step-size controller during the verification runs produced
bit-identical results.

All C harnesses below are compiled from
`C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust\porttest`
with this exact PowerShell command shape (substitute the file name):

```
cd C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust\porttest
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && cl /nologo /I"..\..\rebound\rebound\src" /D_GNU_SOURCE /D_CRT_SECURE_NO_WARNINGS /D_CRT_NONSTDC_NO_WARNINGS /Ox /fp:precise integrators_test.c librebound.lib /Fe:integrators_test.exe'
```

### 7.1 Integrator matrix — `integrators_test`

Three-body problem (star m=1; planet m=1e-3 at x=1.6, vy=0.5; moon m=1e-7 at
x=1.7, vy=0.6, z=0.01, vz=0.001), G=1, dt=0.01, 500 fixed steps, final state
bit-dumped. Run (from `porttest`):

```
.\integrators_test.exe <config> 2 500
..\target\release\examples\integrators_test.exe <config> 2 500
```

then `Compare-Object (gc state_c_final.txt) (gc state_rust_final.txt)`.

**Result: 63 of 63 configurations bit-identical** —

| Integrator | Configurations (all IDENTICAL) |
|---|---|
| none | none |
| ias15 | ias15 (1000-step adaptive run additionally verified) |
| leapfrog | orders 2, 4, 6, 8 |
| whfast | default, c11, c17(+corrector2), dh, whds, bary, mk, comp, lazy, usafe |
| saba | default, 1, 2, 3, 4, cm2, cl2, 104, 864, h844, h864, h1064, usafe |
| janus | default(6), 2, 4, 8, 10 |
| eos | default, all nine phi0==phi1 diagonals, 2-7, 5-8, usafe |
| mercurius | default, usafe, c4, c5, inf, hill01 — the default config keeps the moon inside the planet's critical radius, so the IAS15 close-encounter machinery runs **every** step (5000-step run additionally verified) |
| bs | default, tight (1e-11), loose (1e-6), maxdt |
| trace | default, pbs, ias15, hill1, perinone, eta001 — configs were cross-checked to genuinely take the BS-encounter and pericenter FULL paths |

### 7.2 Shearing sheet — `problem_test.c` / `shearing_sheet_test.rs`

The stock Saturn's-rings example (SEI integrator, octree gravity, tree
collision search, shear-periodic boundary, hard-sphere collisions with the
Bridges restitution law, glibc-`rand_r` initial conditions), seed 42,
1482 particles, 400 timesteps, ~10^5 collisions resolved.

```
.\rebound_test.exe 400
..\target\release\examples\shearing_sheet_test.exe 400
Get-FileHash state_c_final.txt   # 75BDAAB7109F125192F56AEB0CCDCAC554AF88A165FE11075EC5A871178521F0
Get-FileHash state_rust_final.txt # identical
```

**Result: byte-identical dumps, identical SHA-256** for both the initial
(post-RNG) and the 400-step final state. (The one controlled change vs the
stock problem.c: the Bridges law `0.32*pow(v*100,-0.234)` is written as
`0.32*exp(-0.234*log(v*100))` — identically on BOTH sides — to keep the
documented `pow` divergence out of the harness; the full history of isolating
`pow` via bisection to a single collision pair is in
`shearing_sheet_port_test.md`, which repeats all commands.)

### 7.3 Orbital derivatives — `derivatives_test`

All 65 `reb_particle_derivative_*` functions over two particle/primary
configurations. **Result: 130/130 output lines bit-identical.**

### 7.4 Frequency analysis — `frequency_test`

Three-frequency synthetic complex signal, 256 samples; MFT, FMFT and FMFT2
modes; frequencies, amplitudes and phases bit-dumped. **Result: all three
modes bit-identical** (this exercises the FFT, Hanning window,
golden-section maximization, Gram-Schmidt orthogonalization and the
amplitude sort end to end).

### 7.5 Simulationarchive cross-language round trips — `archive_test`

The strongest interoperability test. In each direction, one implementation
runs 3×100 steps saving a Simulationarchive snapshot after each 100; the
*other* implementation loads snapshot 1 (the 200-step state) from that
archive, continues 100 more steps, and must land on the 300-step state of
the writer **bit-exactly**:

```
.\archive_test.exe whfast-usafe write        # C writes archive_c_whfast-usafe.bin
..\target\release\examples\archive_test.exe whfast-usafe continue   # Rust loads + continues
..\target\release\examples\archive_test.exe whfast-usafe write      # Rust writes archive_rust_...bin
.\archive_test.exe whfast-usafe continue     # C loads + continues
```

**Result: IDENTICAL in all four directions × two integrators** —
`whfast-usafe` (round-trips the unsynchronized Jacobi coordinates `p_jh`)
and `ias15` (round-trips the adaptive-step restart arrays `br`/`er`). This
proves the binary format, including the incremental diff-blob append
mechanism, is fully compatible between the MSVC C build and the Rust port.

### 7.6 Web server — `server_test`

The Rust example pauses a 100-step simulation and serves it; the blob
fetched from the running Rust server is then loaded by the C build:

```
Start-Process ..\target\release\examples\server_test.exe   # serves on port 12873
curl.exe -s http://localhost:12873/simulation --output served.bin
curl.exe -s http://localhost:12873/keyboard/81             # 'Q' = quit
.\archive_test.exe whfast load served.bin
```

**Result: the C build loads the HTTP-served blob to the bit-identical
state**, and the keyboard endpoint cleanly resumed/terminated the paused
Rust simulation.

### 7.7 add_fmt / datasets — `addfmt_test`

Built-in "solar system" dataset plus particles from orbital elements, Pal
coordinates, and an orbital period. **Result: all 12 particles bit-identical.**

## 8. Building the Rust crate — all cargo commands

From `C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust`:

```
cargo build --release
cargo build --release --example shearing_sheet
cargo build --release --example shearing_sheet_test
cargo build --release --example integrators_test
cargo build --release --example libm_diff
cargo build --release --example derivatives_test
cargo build --release --example frequency_test
cargo build --release --example archive_test
cargo build --release --example server_test
cargo build --release --example addfmt_test
```

Every build completes with **zero warnings** under `#![forbid(unsafe_code)]`
and `#![deny(warnings)]`. There are no dependencies to fetch; the crate is
std-only and builds offline. (A plain `cargo build` produces the debug
profile; all verification used `--release`, and release/debug produce
identical floating-point results since no fast-math or FMA contraction is
enabled in either — confirmed by `reb_check_fp_contract()` returning 0.)

## 9. Known limitations

1. **`pow`**: the single libm function where Rust does not match the UCRT
   (≤2 ulp, 0.03% of sampled inputs). REBOUND core integration paths do not
   call `pow` per step; it appears in the BS step-size controller (all calls
   observed in testing matched bitwise), in `reb_random_powerlaw`, and in
   user problem code (e.g. the stock Bridges law). A run whose trajectory
   passes through one of the rare divergent inputs can differ in final bits
   from the C build while remaining correct to ≤2 ulp at the divergence
   point.
2. **WHFast512** integrates nothing on Windows — in C and Rust alike; both
   produce the identical "AVX512 is not supported on your platform." error.
3. Excluded subsystems: OpenGL display, MPI (§4.1).
4. The C's documented restrictions carry over unchanged (e.g. MERCURIUS/TRACE
   emit the same warnings for variational equations, collision-search modes,
   gravity-routine overrides).

## 10. Using the pure Rust code

### 10.1 As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
rebound_rs = { path = "C:/Users/nsh/Developer/github/rustSolveIt_Win11_SUNDIALS_7_8_0/rebound_rust" }
```

Minimal three-body integration (names are the C names):

```rust
use rebound_rs::*;

fn main() {
    let mut sim = reb_simulation_create();
    let r = &mut sim;
    reb_simulation_set_integrator(r, "whfast");   // "ias15" (default), "whfast",
                                                  // "saba", "janus", "eos", "sei",
                                                  // "leapfrog", "mercurius", "bs",
                                                  // "trace", "none", "whfast512"
    r.G = 1.0;
    r.dt = 0.01;

    let mut star = reb_particle::default();
    star.m = 1.0;
    reb_simulation_add(r, star);

    // Either explicit Cartesian state...
    let mut planet = reb_particle::default();
    planet.m = 1e-3; planet.x = 1.6; planet.vy = 0.5;
    reb_simulation_add(r, planet);

    // ...or orbital elements via add_fmt (values as a slice, in token order):
    reb_simulation_add_fmt(r, "m a e", &[
        reb_fmt_arg::d(1e-7), reb_fmt_arg::d(2.5), reb_fmt_arg::d(0.1),
    ]);

    reb_simulation_move_to_com(r);
    reb_simulation_integrate(r, 100.0);           // or reb_simulation_steps(r, 10_000)

    println!("t = {}", r.t);
    for i in 0..r.N {
        let p = r.particles[i];
        println!("{} {} {} {}", i, p.x, p.y, p.z);
    }
    println!("E = {}", reb_simulation_energy(r));
}
```

Integrator options are set on the state enum, e.g.:

```rust
if let reb_integrator_state::whfast(ref mut wh) = r.integrator {
    wh.corrector = 17;
    wh.safe_mode = 0;
}
```

Saving/continuing with Simulationarchives (files are interchangeable with
C-REBOUND):

```rust
reb_simulation_save_to_file(r, Some("archive.bin"));            // save/append snapshot
reb_simulation_save_to_file_interval(r, "auto.bin", 10.0);      // auto-snapshot every 10 time units
let mut restored = reb_simulation_create_from_file("archive.bin", -1).unwrap(); // -1 = latest
reb_simulation_integrate(&mut restored, 200.0);
```

Browser visualization (needs `rebound.html`, auto-downloaded on first start):

```rust
reb_simulation_start_server(r, 1234);   // then open http://localhost:1234
reb_simulation_integrate(r, 1e6);
reb_simulation_stop_server(r);
```

### 10.2 Running the examples

```
cd C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust
cargo run --release --example shearing_sheet
cargo run --release --example integrators_test -- whfast 2 500
cargo run --release --example shearing_sheet_test -- 400
cargo run --release --example derivatives_test
cargo run --release --example frequency_test
cargo run --release --example archive_test -- whfast-usafe write
cargo run --release --example server_test
cargo run --release --example addfmt_test
cargo run --release --example libm_diff
```

Each example has a companion Jupyter notebook in `notebooks\` (one per
example) that builds it, runs it, and displays/plots the output; start with:

```
cd C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust\notebooks
jupyter lab
```

### 10.3 Reproducing the verification

1. Build the C reference (§3).
2. Build the Rust examples (§8).
3. Compile the C harnesses in `porttest\` with the command in §7.
4. Run the matched pairs and `Compare-Object` the dumps as shown in
   §7.1–§7.7.

## 11. Licensing

REBOUND is (c) Hanno Rein, Shangfei Liu, Dave O'Hallaron, Ernst Hairer,
Tiger Lu, Dan Tamayo, Rishit Dagli, Pejvak Javaheri and contributors,
GPL-3.0-or-later. This Rust translation is a derivative work distributed
under the same license; every module header cites the C file it translates
and its copyright holders. The upstream commit translated is
`dad5f97806ecbb408dcaff728851c64e67f9f6eb` (version string "5.1.1", carried
in the crate as `reb_version_str` and `reb_githash_str`).
