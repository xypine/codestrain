use std::collections::{HashMap, VecDeque};

use codestrain_common::*;
use extism_pdk::*;

struct StackEntry {
    first: (i32, i32),
    pos: (i32, i32),
    distance: usize,
}

#[plugin_fn]
pub fn take_turn(Json(input): Json<StrainInput>) -> FnResult<Json<StrainOutput>> {
    let board = input.board;
    // find the cell with the maximum x and y
    let target = board.iter().max_by_key(|((x, y), _)| x + y).unwrap().0;
    let mut stack: VecDeque<StackEntry> = input
        .allowed
        .iter()
        .map(|&first| StackEntry {
            first,
            pos: first,
            distance: 0,
        })
        .collect();
    let mut visited: HashMap<(i32, i32), usize> = HashMap::new();
    let mut closest: Option<(i32, (i32, i32))> = None;
    while let Some(current) = stack.pop_front() {
        if current.pos == target {
            return Ok(Json(current.first));
        }
        let distance_to_target =
            (target.0 - current.pos.0).abs() + (target.1 - current.pos.1).abs();
        if let Some((d, _)) = closest {
            if distance_to_target < d {
                closest = Some((distance_to_target, current.first));
            }
        } else {
            closest = Some((distance_to_target, current.first));
        }
        if let Some(&distance) = visited.get(&current.pos) {
            if distance <= current.distance {
                continue;
            }
        }
        visited.insert(current.pos, current.distance);
        for &(dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (current.pos.0 + dx, current.pos.1 + dy);
            let cell = board
                .iter()
                .find(|&((x, y), _)| *x == next.0 && *y == next.1)
                .map(|(_, b)| b);
            match cell {
                None => continue, // cell is out of bounds
                Some(None) => {}  // cell is empty
                _ => continue,    // cell is blocked
            }
            stack.push_back(StackEntry {
                first: current.first,
                pos: next,
                distance: current.distance + 1,
            });
        }
    }
    // no path found, return the closest cell
    Ok(Json(closest.unwrap().1))
}
