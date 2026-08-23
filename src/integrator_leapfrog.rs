//! integrator_leapfrog.rs — drift-kick-drift leapfrog.
//! Phase-B module: the full translation of integrator_leapfrog.c lands
//! with the remaining integrator family; until then stepping with
//! "leapfrog" reports an explicit error instead of inventing numerics
//! (porting rule: missing symbols are reported, never invented).
//!
//! Part of rebound_rs, a GPL-3.0-or-later translation of REBOUND 5.1.1
//! (c) Hanno Rein and contributors. See crate root.

use crate::tools::reb_simulation_error;
use crate::types::*;

pub fn reb_integrator_leapfrog_step(r: &mut reb_simulation) {
    reb_simulation_error(
        r,
        "Integrator 'leapfrog' is not yet ported in this phase of rebound_rs (C source: src/integrator_leapfrog.c).",
    );
    r.status = REB_STATUS_GENERIC_ERROR;
}
