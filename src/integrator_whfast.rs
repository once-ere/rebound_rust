//! integrator_whfast.rs — WHFast (Wisdom-Holman with corrector family).
//! Phase-B module: the full translation of integrator_whfast.c lands
//! with the remaining integrator family; until then stepping with
//! "whfast" reports an explicit error instead of inventing numerics
//! (porting rule: missing symbols are reported, never invented).
//!
//! Part of rebound_rs, a GPL-3.0-or-later translation of REBOUND 5.1.1
//! (c) Hanno Rein and contributors. See crate root.

use crate::tools::reb_simulation_error;
use crate::types::*;

/// Configuration/state of WHFast (subset carried until the Phase-B port).
#[derive(Clone, Debug, Default)]
pub struct reb_integrator_whfast_state {
    pub corrector: u32,
    pub corrector2: u32,
    pub kernel: u32,
    pub coordinates: u32,
    pub recalculate_coordinates_this_timestep: u32,
    pub safe_mode: u32,
    pub keep_unsynchronized: u32,
    pub is_synchronized: u32,
    pub timestep_warning: u32,
    pub recalculate_coordinates_but_not_synchronized_warning: u32,
}

pub fn reb_integrator_whfast_step(r: &mut reb_simulation) {
    reb_simulation_error(
        r,
        "Integrator 'whfast' is not yet ported in this phase of rebound_rs (C source: src/integrator_whfast.c).",
    );
    r.status = REB_STATUS_GENERIC_ERROR;
}
