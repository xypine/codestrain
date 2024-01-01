use std::collections::HashSet;

use extism_pdk::*;

#[plugin_fn]
pub fn take_turn(board_str: &str) -> FnResult<String> {
    let board: HashSet<(u32, u32)> = serde_json::from_str(board_str).expect("invalid board");
    Ok(format!("Hello, {}!", name))
}
