use icarus_derive::*;
use serde::{Deserialize, Serialize};
use candid::CandidType;

#[derive(IcarusStorable, Debug, Clone, Serialize, Deserialize, CandidType)]
struct TestData {
    name: String,
    value: u64,
}

fn main() {}