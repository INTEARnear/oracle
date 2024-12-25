mod tests;

use tokio::sync::OnceCell;

static CONTRACT_WASM: OnceCell<Vec<u8>> = OnceCell::const_new();

pub async fn get_contract_wasm() -> &'static Vec<u8> {
    CONTRACT_WASM
        .get_or_init(|| async {
            near_workspaces::compile_project("./")
                .await
                .expect("compiling `intear-oracle` contract for tests")
        })
        .await
}

static FT_CONTRACT_WASM: OnceCell<Vec<u8>> = OnceCell::const_new();

pub async fn get_ft_contract_wasm() -> &'static Vec<u8> {
    FT_CONTRACT_WASM
        .get_or_init(|| async {
            near_workspaces::compile_project("../test-ft-contract")
                .await
                .expect("compiling `test-ft-contract` contract for tests")
        })
        .await
}

static EXMAPLE_CONSUMER_CONTRACT_WASM: OnceCell<Vec<u8>> = OnceCell::const_new();

pub async fn get_example_consumer_contract_wasm() -> &'static Vec<u8> {
    EXMAPLE_CONSUMER_CONTRACT_WASM
        .get_or_init(|| async {
            near_workspaces::compile_project("../example-consumer")
                .await
                .expect("compiling `example-consumer` contract for tests")
        })
        .await
}
