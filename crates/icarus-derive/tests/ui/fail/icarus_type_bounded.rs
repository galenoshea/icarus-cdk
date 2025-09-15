use icarus_derive::*;
use serde::{Deserialize, Serialize};
use candid::CandidType;

#[derive(IcarusType, Debug, Clone, Serialize, Deserialize, CandidType)]
#[icarus_storable(bounded, max_size = "1MB")]
struct BoundedData {
    content: String,
    timestamp: u64,
}

fn main() {}