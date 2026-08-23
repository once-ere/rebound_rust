//! integrator_ias15.rs — IAS15 (15th-order adaptive integrator).
//! Phase-B module: the full translation of integrator_ias15.c lands
//! with the remaining integrator family; until then stepping with
//! "ias15" reports an explicit error instead of inventing numerics
//! (porting rule: missing symbols are reported, never invented).
//!
//! Part of rebound_rs, a GPL-3.0-or-later translation of REBOUND 5.1.1
//! (c) Hanno Rein and contributors. See crate root.

use crate::tools::reb_simulation_error;
use crate::types::*;

/// Configuration/state of IAS15 (subset carried until the Phase-B port).
#[derive(Clone, Debug, Default)]
pub struct reb_integrator_ias15_state {
    pub epsilon: f64,
    pub min_dt: f64,
    pub adaptive_mode: u32,
}

pub fn reb_integrator_ias15_step(r: &mut reb_simulation) {
    reb_simulation_error(
        r,
        "Integrator 'ias15' is not yet ported in this phase of rebound_rs (C source: src/integrator_ias15.c).",
    );
    r.status = REB_STATUS_GENERIC_ERROR;
}
