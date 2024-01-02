use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrainInput {
    pub board: Vec<((i32, i32), Option<bool>)>,
    pub allowed: Vec<(i32, i32)>,
}

pub type StrainOutput = (i32, i32);
