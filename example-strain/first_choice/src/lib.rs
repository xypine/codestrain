use codestrain_common::*;
use extism_pdk::*;

#[plugin_fn]
pub fn take_turn(Json(input): Json<StrainInput>) -> FnResult<Json<StrainOutput>> {
    let allowed = input.allowed;
    let first = allowed.first().expect("No allowed moves!");
    Ok(Json(*first))
}
