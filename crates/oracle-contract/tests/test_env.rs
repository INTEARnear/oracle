mod tests;

use std::sync::LazyLock;

use near_workspaces::cargo_near_build;

pub static CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let artifact =
        cargo_near_build::build(Default::default()).expect("building `oracle` contract for tests");
    let contract_wasm = std::fs::read(&artifact.path)
        .map_err(|err| format!("accessing {} to read wasm contents: {}", artifact.path, err))
        .expect("std::fs::read");
    contract_wasm
});
