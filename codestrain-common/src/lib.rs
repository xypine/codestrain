use std::collections::{HashMap, HashSet};

pub struct StrainInput {
    pub board: HashMap<(u32, u32), Option<bool>>,
    pub allowed: HashSet<(u32, u32)>,
}

pub type StrainOutput = (u32, u32);
