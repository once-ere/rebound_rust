//! rebound_rs — a pure-Rust translation of REBOUND 5.1.1
//! (github.com/hannorein/rebound @ dad5f978, "Patch (#931)").
//!
//! REBOUND is an open-source multi-purpose N-body code by Hanno Rein and
//! collaborators, licensed under the GNU General Public License v3 (or
//! later). This translation is a derivative work under the same license;
//! see the LICENSE file carried in this crate. Original authors and
//! copyright holders: Hanno Rein, Shangfei Liu, and the REBOUND
//! contributors.
//!
//! Translation rules (mirroring the sundials_rs porting discipline):
//! - zero `unsafe`, zero external dependencies, zero warnings;
//! - C function and struct names are preserved (`reb_simulation_create`,
//!   `reb_particle`, ...) — the crate root allows the C spellings;
//! - control flow, constants and arithmetic ORDER match the C source
//!   line for line (floating point is not associative);
//! - the glibc `rand_r` generator is reproduced exactly, so random
//!   initial conditions are bit-identical to the C build's;
//! - C's malloc'd pointer graphs become owned Rust containers: the
//!   particle array is a `Vec<reb_particle>`, the octree an index
//!   arena rebuilt each use exactly as the C rebuilds its cells.
//!
//! Deviations from C, all mechanical, are documented in
//! `rebound_rust.md` §"Deviation classes".

#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod types;
pub mod tools;
pub mod boundary;
pub mod tree;
pub mod gravity;
pub mod collision;
pub mod particle;
pub mod simulation;
pub mod output;
pub mod transformations;
pub mod rotations;
pub mod integrator_none;
pub mod integrator_sei;
pub mod integrator_leapfrog;
pub mod integrator_ias15;
pub mod integrator_whfast;
pub mod integrator_saba;
pub mod integrator_janus;
pub mod integrator_eos;
pub mod integrator_mercurius;
pub mod integrator_bs;
pub mod integrator_trace;
pub mod integrator_whfast512;
pub mod derivatives;
pub mod frequency_analysis;
pub mod binarydata;
pub mod simulationarchive;

pub use types::*;
pub use tools::*;
pub use boundary::*;
pub use gravity::*;
pub use collision::*;
pub use particle::*;
pub use simulation::*;
pub use output::*;
pub use transformations::*;
pub use rotations::*;
pub use derivatives::*;
pub use frequency_analysis::*;
pub use binarydata::*;
pub use simulationarchive::*;

/// Version of the C release this crate translates (rebound.c `reb_version_str`).
pub const reb_version_str: &str = "5.1.1";
/// Git hash of the C source tree the translation was made from.
pub const reb_githash_str: &str = "dad5f97806ecbb408dcaff728851c64e67f9f6eb";
