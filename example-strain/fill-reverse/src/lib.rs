use codestrain_common::*;
use extism_pdk::*;

#[plugin_fn]
pub fn take_turn(Json(input): Json<StrainInput>) -> FnResult<Json<StrainOutput>> {
    // find the option with the max euclidean distance
    let farthest = input
        .allowed
        .iter()
        .max_by_key(|(x, y)| (x.pow(2) + y.pow(2)))
        .unwrap();

    Ok(Json(*farthest))
}
