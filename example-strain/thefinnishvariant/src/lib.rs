use codestrain_common::*;
use extism_pdk::*;

const SEACH_RADIUS: i32 = 10;

#[plugin_fn]
pub fn take_turn(Json(input): Json<StrainInput>) -> FnResult<Json<StrainOutput>> {
    // find the option with the least neighbors within a 2-tile radius
    let best = input
        .allowed
        .iter()
        .min_by_key(|(x, y)| {
            let mut neighbors = 0.0;
            for dx in -SEACH_RADIUS..=SEACH_RADIUS {
                for dy in -SEACH_RADIUS..=SEACH_RADIUS {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let cell = input
                        .board
                        .iter()
                        .find(|c| c.0 .0 == x.wrapping_add(dx) && c.0 .1 == y.wrapping_add(dy));
                    if let Some(cell) = cell {
                        if cell.1 != None {
                            let dist = (dx * dx + dy * dy) as f64;
                            neighbors += 1.0 / dist;
                        }
                    }
                }
            }
            let cmp = (neighbors * 100000.0) as i32;
            cmp
        })
        .unwrap();

    Ok(Json(*best))
}
