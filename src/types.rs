//! types.rs — the REBOUND data structures (translated from rebound.h).
//!
//! Part of rebound_rs, a GPL-3.0-or-later translation of REBOUND 5.1.1
//! (c) Hanno Rein, Shangfei Liu and contributors. See crate root.

/// One particle (rebound.h `struct reb_particle`).
///
/// Deviation: the C struct's `name` (interned `const char*`), `ap`
/// (REBOUNDx attachment) and `sim` (parent back-pointer) cannot exist in
/// safe owned Rust. `name` becomes an index into the simulation's
/// `name_list`; `ap` has no equivalent (REBOUNDx is C-only); functions
/// that used the `sim` back-pointer take the simulation explicitly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct reb_particle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub ax: f64,
    pub ay: f64,
    pub az: f64,
    pub m: f64,
    pub r: f64,
    /// Index into `reb_simulation::name_list`; `None` = unnamed.
    pub name: Option<usize>,
}

impl Default for reb_particle {
    fn default() -> Self {
        reb_particle {
            x: 0., y: 0., z: 0.,
            vx: 0., vy: 0., vz: 0.,
            ax: 0., ay: 0., az: 0.,
            m: 0., r: 0.,
            name: None,
        }
    }
}

/// Generic 3d vector (rebound.h `struct reb_vec3d`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct reb_vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Generic 6d vector (rebound.h `struct reb_vec6d`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct reb_vec6d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

/// One particle-particle collision (rebound.h `struct reb_collision`).
/// `usize::MAX` plays the role of C's `SIZE_MAX` sentinel.
#[derive(Clone, Copy, Debug)]
pub struct reb_collision {
    pub p1: usize,
    pub p2: usize,
    pub gb: reb_vec6d,
    pub ri: usize,
}

/// Return values for collision resolve functions
/// (rebound.h `enum REB_COLLISION_RESOLVE_OUTCOME`). Bit flags.
pub type REB_COLLISION_RESOLVE_OUTCOME = i32;
pub const REB_COLLISION_RESOLVE_OUTCOME_REMOVE_NONE: i32 = 0;
pub const REB_COLLISION_RESOLVE_OUTCOME_REMOVE_P1: i32 = 1;
pub const REB_COLLISION_RESOLVE_OUTCOME_REMOVE_P2: i32 = 2;
pub const REB_COLLISION_RESOLVE_OUTCOME_REMOVE_BOTH: i32 = 3;

/// Possible values of `reb_simulation::status`
/// (rebound.h `enum REB_STATUS`). Kept as an integer because the C code
/// increments statuses below `SINGLE_STEP` once per timestep.
pub type REB_STATUS = i32;
pub const REB_STATUS_SINGLE_STEP: i32 = -10;
pub const REB_STATUS_SCREENSHOT_READY: i32 = -5;
pub const REB_STATUS_SCREENSHOT: i32 = -4;
pub const REB_STATUS_PAUSED: i32 = -3;
pub const REB_STATUS_LAST_STEP: i32 = -2;
pub const REB_STATUS_RUNNING: i32 = -1;
pub const REB_STATUS_SUCCESS: i32 = 0;
pub const REB_STATUS_GENERIC_ERROR: i32 = 1;
pub const REB_STATUS_NO_PARTICLES: i32 = 2;
pub const REB_STATUS_ENCOUNTER: i32 = 3;
pub const REB_STATUS_ESCAPE: i32 = 4;
pub const REB_STATUS_USER: i32 = 5;
pub const REB_STATUS_SIGINT: i32 = 6;
pub const REB_STATUS_COLLISION: i32 = 7;

/// Gravity ignore-terms flag (anonymous enum in rebound.h).
pub type REB_GRAVITY_IGNORE_TERMS = u32;
pub const REB_GRAVITY_IGNORE_TERMS_NONE: u32 = 0;
pub const REB_GRAVITY_IGNORE_TERMS_BETWEEN_0_AND_1: u32 = 1;
pub const REB_GRAVITY_IGNORE_TERMS_INVOLVING_0: u32 = 2;

/// Collision module selection (anonymous enum in rebound.h).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum REB_COLLISION {
    NONE = 0,
    DIRECT = 1,
    TREE = 2,
    LINE = 4,
    LINETREE = 5,
}

/// Boundary module selection (anonymous enum in rebound.h).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum REB_BOUNDARY {
    NONE = 0,
    OPEN = 1,
    PERIODIC = 2,
    SHEAR = 3,
}

/// Gravity module selection (anonymous enum in rebound.h).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum REB_GRAVITY {
    NONE = 0,
    BASIC = 1,
    COMPENSATED = 2,
    TREE = 3,
    JACOBI = 5,
    CUSTOM = 7,
}

/// One node of the collision/gravity octree (tree.h `struct
/// reb_treecell`), arena-allocated: `oct` holds indices into
/// `reb_simulation::tree_cells` instead of pointers. `usize::MAX`
/// encodes the C NULL child.
#[derive(Clone, Copy, Debug)]
pub struct reb_treecell {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
    pub m: f64,
    pub mx: f64,
    pub my: f64,
    pub mz: f64,
    pub oct: [usize; 8],
    /// Leaf: the particle index (>= 0). Non-leaf: -(number of
    /// particles in this cell), exactly the C encoding.
    pub pt: i32,
    /// MPI essential-tree flag; always 0 here (MPI excluded).
    pub remote: i32,
}

pub const REB_TREECELL_NONE: usize = usize::MAX;

/// Orbital elements (rebound.h `struct reb_orbit`).
#[derive(Clone, Copy, Debug, Default)]
pub struct reb_orbit {
    pub d: f64,
    pub v: f64,
    pub h: f64,
    pub P: f64,
    pub n: f64,
    pub a: f64,
    pub e: f64,
    pub inc: f64,
    pub Omega: f64,
    pub omega: f64,
    pub pomega: f64,
    pub f: f64,
    pub M: f64,
    pub l: f64,
    pub theta: f64,
    pub T: f64,
    pub rhill: f64,
    pub pal_h: f64,
    pub pal_k: f64,
    pub pal_ix: f64,
    pub pal_iy: f64,
    pub hvec: reb_vec3d,
    pub evec: reb_vec3d,
}

/// Rotation, implemented as a quaternion (rebound.h `struct reb_rotation`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct reb_rotation {
    pub ix: f64,
    pub iy: f64,
    pub iz: f64,
    pub r: f64,
}

/// A variational-equation configuration
/// (rebound.h `struct reb_variational_configuration`, minus the C
/// back-pointer to the simulation).
#[derive(Clone, Copy, Debug)]
pub struct reb_variational_configuration {
    pub order: i32,
    pub index: usize,
    pub testparticle: i32,
    pub index_1st_order_a: usize,
    pub index_1st_order_b: usize,
    pub lrescale: f64,
}

/// The per-integrator state (C: `void* state` behind the
/// `reb_integrator` vtable; Rust: one enum variant per built-in).
#[derive(Clone, Debug)]
pub enum reb_integrator_state {
    none,
    sei(crate::integrator_sei::reb_integrator_sei_state),
    leapfrog(crate::integrator_leapfrog::reb_integrator_leapfrog_state),
    ias15(crate::integrator_ias15::reb_integrator_ias15_state),
    whfast(crate::integrator_whfast::reb_integrator_whfast_state),
}

impl reb_integrator_state {
    pub fn name(&self) -> &'static str {
        match self {
            reb_integrator_state::none => "none",
            reb_integrator_state::sei(_) => "sei",
            reb_integrator_state::leapfrog(_) => "leapfrog",
            reb_integrator_state::ias15(_) => "ias15",
            reb_integrator_state::whfast(_) => "whfast",
        }
    }
}

/// Message kinds (rebound_internal.h `enum REB_MESSAGE_TYPE`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum REB_MESSAGE_TYPE {
    INFO,
    ERROR,
    WARNING,
}

pub const REB_STRING_SIZE_MAX: usize = 256;
pub const reb_messages_max_N: usize = 10;

/// Main REBOUND simulation structure (rebound.h `struct reb_simulation`).
///
/// Pointer-based C members become owned Rust containers; members that
/// belong to the excluded subsystems (MPI buffers, OpenGL display
/// data) are not carried — the full accounting is in rebound_rust.md.
pub struct reb_simulation {
    pub t: f64,
    pub G: f64,
    pub softening: f64,
    pub OMEGA: f64,
    pub OMEGAZ: f64,
    pub dt: f64,
    pub dt_last_done: f64,
    pub steps_done: u64,
    pub is_synchronized: u32,
    pub did_modify_particles: u32,

    // Main particles array. N == particles.len() is maintained
    // explicitly to mirror the C bookkeeping (N_allocated is the Vec's
    // capacity concern and not carried).
    pub N: usize,
    pub particles: Vec<reb_particle>,

    pub N_map: usize,
    pub map: Option<Vec<usize>>,

    pub N_var: usize,
    pub particles_var: Vec<reb_particle>,
    pub var_config: Vec<reb_variational_configuration>,

    pub N_active: usize,
    pub testparticle_type: i32,
    pub testparticle_hidewarnings: i32,
    pub name_list: Vec<String>,

    pub gravity_cs: Vec<reb_vec3d>,
    /// Octree roots: one entry per root box; REB_TREECELL_NONE = C NULL.
    pub tree_root: Vec<usize>,
    /// Octree cell arena (C: individually malloc'd reb_treecell).
    pub tree_cells: Vec<reb_treecell>,
    pub opening_angle2: f64,
    pub status: REB_STATUS,
    pub exact_finish_time: i32,

    pub force_is_velocity_dependent: i32,
    pub gravity_ignore_terms: REB_GRAVITY_IGNORE_TERMS,
    pub output_timing_last: f64,

    pub save_messages: i32,
    pub messages: Vec<(REB_MESSAGE_TYPE, String)>,

    pub messages_var_rescale_warning: i32,
    pub messages_timestep_warning: i32,

    pub exit_max_distance: f64,
    pub exit_min_distance: f64,
    pub usleep: f64,
    pub track_energy_offset: i32,
    pub energy_offset: f64,
    pub walltime: f64,
    pub walltime_last_step: f64,
    pub walltime_last_steps: f64,
    pub walltime_last_steps_sum: f64,
    pub walltime_last_steps_N: i32,

    // Simulation domain and ghost boxes
    pub root_size: f64,
    pub N_root_x: usize,
    pub N_root_y: usize,
    pub N_root_z: usize,
    pub N_ghost_x: i32,
    pub N_ghost_y: i32,
    pub N_ghost_z: i32,

    // Collision related variables
    pub collisions: Vec<reb_collision>,
    pub N_collisions: usize,
    pub N_targets: usize,
    pub minimum_collision_velocity: f64,
    pub collisions_plog: f64,
    pub collisions_log_n: i64,

    // MEGNO
    pub calculate_megno: i32,
    pub megno_Ys: f64,
    pub megno_Yss: f64,
    pub megno_cov_Yt: f64,
    pub megno_var_t: f64,
    pub megno_mean_t: f64,
    pub megno_mean_Y: f64,
    pub megno_initial_t: f64,
    pub megno_n: i64,

    /// Seed for the glibc-compatible `rand_r` generator (tools.rs).
    pub rand_seed: u32,

    // Module selection
    pub collision: REB_COLLISION,
    pub boundary: REB_BOUNDARY,
    pub gravity: REB_GRAVITY,
    pub integrator: reb_integrator_state,

    pub gravity_custom: Option<fn(&mut reb_simulation)>,

    // Callback functions (plain fn pointers, like the C).
    pub additional_forces: Option<fn(&mut reb_simulation)>,
    pub pre_timestep_modifications: Option<fn(&mut reb_simulation)>,
    pub post_timestep_modifications: Option<fn(&mut reb_simulation)>,
    pub heartbeat: Option<fn(&mut reb_simulation)>,
    pub coefficient_of_restitution: Option<fn(&reb_simulation, f64) -> f64>,
    pub collision_resolve:
        Option<fn(&mut reb_simulation, reb_collision) -> REB_COLLISION_RESOLVE_OUTCOME>,
}
