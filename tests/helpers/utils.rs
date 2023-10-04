use std::fmt::Display;

use atomic_swaps_contract::msg::InstantiateMsg;
use osmosis_test_tube::{OsmosisTestApp, RunnerError, SigningAccount, Wasm};

pub fn wasm_file() -> Vec<u8> {
    let wasm_file_path = format!("./artifacts/atomic_swaps_contract");

    match std::fs::read(wasm_file_path.clone()) {
        Ok(bytes) => {
            println!("{wasm_file_path}.wasm");
            bytes
        }
        // Retry if in arch64 environment
        Err(_) => {
            let alt_file_path = format!("{wasm_file_path}-aarch64.wasm");
            println!("{}", alt_file_path);
            std::fs::read(alt_file_path).unwrap()
        }
    }
}

pub fn instantiate_contract(wasm: &Wasm<OsmosisTestApp>, owner: &SigningAccount) -> String {
    let code_id = wasm
        .store_code(&wasm_file(), None, owner)
        .unwrap()
        .data
        .code_id;

    wasm.instantiate(
        code_id,
        &InstantiateMsg {},
        None,
        Some("atomic-swaps-contract"),
        &[],
        owner,
    )
    .unwrap()
    .data
    .address
}

pub fn assert_err(actual: RunnerError, expected: impl Display) {
    match actual {
        RunnerError::ExecuteError { msg } => {
            println!("ExecuteError, msg: {msg}");
            assert!(msg.contains(&format!("{expected}")))
        }
        RunnerError::QueryError { msg } => {
            println!("QueryError, msg: {msg}");
            assert!(msg.contains(&format!("{expected}")))
        }
        _ => panic!("Unhandled error"),
    }
}
