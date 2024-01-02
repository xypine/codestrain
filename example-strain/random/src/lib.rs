use std::collections::HashSet;

use codestrain_common::*;
use extism_pdk::*;

#[plugin_fn]
pub fn take_turn(Json(input): Json<StrainInput>) -> FnResult<Json<StrainOutput>> {
    let allowed: HashSet<(i32, i32)> = HashSet::from_iter(input.allowed);
    let first = allowed.iter().next().expect("No allowed moves!");
    Ok(Json(*first))
}
