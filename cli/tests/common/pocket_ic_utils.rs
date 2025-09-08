//! PocketIC test utilities for Icarus authentication testing

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use std::path::PathBuf;
use std::process::Command;

/// Create and setup a PocketIC instance
pub fn setup_pocket_ic() -> PocketIc {
    PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build()
}

/// Deploy a test canister with the specified owner
pub async fn deploy_test_canister(
    pic: &PocketIc,
    wasm_path: PathBuf,
    owner: Principal,
) -> Principal {
    // Create canister
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000); // 100T cycles

    // Read WASM
    let wasm = std::fs::read(wasm_path).expect("Failed to read WASM file");

    // Install with owner as init argument
    let init_args = encode_args((owner,)).expect("Failed to encode init args");
    pic.install_canister(canister_id, wasm, init_args, None);

    canister_id
}

/// Call a canister method with specific principal
pub fn call_with_principal<T: CandidType>(
    pic: &PocketIc,
    sender: Principal,
    canister_id: Principal,
    method: &str,
    args: T,
) -> Result<Vec<u8>, String> {
    let encoded_args = encode_args((args,)).expect("Failed to encode args");

    match pic.update_call(canister_id, sender, method, encoded_args) {
        Ok(WasmResult::Reply(bytes)) => Ok(bytes),
        Ok(WasmResult::Reject(msg)) => Err(msg),
        Err(e) => Err(format!("Call failed: {:?}", e)),
    }
}

/// Query a canister method with specific principal
pub fn query_with_principal<T: CandidType>(
    pic: &PocketIc,
    sender: Principal,
    canister_id: Principal,
    method: &str,
    args: T,
) -> Result<Vec<u8>, String> {
    let encoded_args = encode_args((args,)).expect("Failed to encode args");

    match pic.query_call(canister_id, sender, method, encoded_args) {
        Ok(WasmResult::Reply(bytes)) => Ok(bytes),
        Ok(WasmResult::Reject(msg)) => Err(msg),
        Err(e) => Err(format!("Query failed: {:?}", e)),
    }
}

/// Authentication test response types
#[derive(Debug, Clone, Deserialize, CandidType)]
pub struct AuthInfo {
    pub principal: String,
    pub role: AuthRole,
    pub is_authenticated: bool,
    pub last_access: Option<u64>,
    pub access_count: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, CandidType)]
pub enum AuthRole {
    Owner,
    Admin,
    User,
    ReadOnly,
}

/// Helper to decode authentication responses
pub fn decode_auth_response(response: Vec<u8>) -> Result<AuthInfo, String> {
    decode_args(&response)
        .map(|(info,)| info)
        .map_err(|e| format!("Failed to decode auth response: {}", e))
}

/// Build test canister WASM
pub fn build_test_canister(project_path: &str) -> PathBuf {
    // Build the canister
    let output = Command::new("cargo")
        .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir(project_path)
        .output()
        .expect("Failed to build test canister");

    if !output.status.success() {
        panic!(
            "Failed to build test canister: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Extract candid and optimize
    let wasm_name = project_path.split('/').last().unwrap().replace('-', "_");
    let wasm_path = PathBuf::from(project_path)
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}.wasm", wasm_name));

    // Run candid-extractor
    let did_path = PathBuf::from(project_path)
        .join("src")
        .join(format!("{}.did", project_path.split('/').last().unwrap()));

    Command::new("candid-extractor")
        .args(&[
            wasm_path.to_str().unwrap(),
            "-c",
            did_path.to_str().unwrap(),
            "-o",
            did_path.to_str().unwrap(),
        ])
        .output()
        .ok(); // Ignore if candid-extractor not available

    // Optimize with ic-wasm
    Command::new("ic-wasm")
        .args(&[
            wasm_path.to_str().unwrap(),
            "-o",
            wasm_path.to_str().unwrap(),
            "shrink",
        ])
        .output()
        .ok(); // Ignore if ic-wasm not available

    wasm_path
}

/// Assert that a call was denied with proper error
pub fn assert_access_denied(result: Result<Vec<u8>, String>) {
    match result {
        Err(msg) => {
            assert!(
                msg.contains("denied") || msg.contains("not authorized") || msg.contains("trap"),
                "Expected access denied error, got: {}",
                msg
            );
        }
        Ok(_) => panic!("Expected access denied, but call succeeded"),
    }
}

/// Assert that a call was granted
pub fn assert_access_granted(result: Result<Vec<u8>, String>) {
    match result {
        Ok(_) => {} // Success
        Err(msg) => panic!("Expected access granted, but got error: {}", msg),
    }
}
