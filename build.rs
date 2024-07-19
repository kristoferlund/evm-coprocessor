use dotenv::dotenv;
use ic_cdk_bindgen::{Builder, Config};
use std::env;
use std::path::PathBuf;

fn main() {
    dotenv().ok();

    env::set_var("CANISTER_ID_EVM_RPC_TYPES", "7hfb6-caaaa-aaaar-qadga-cai");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));

    let evm_rpc_did_path = manifest_dir.join("evm-rpc.did");

    let evm_rpc_did_str = evm_rpc_did_path.to_str().expect("Path invalid");

    env::set_var("CANISTER_CANDID_PATH_EVM_RPC_TYPES", evm_rpc_did_str);

    let mut builder = Builder::new();

    let mut evm_rpc_types = Config::new("evm_rpc_types");
    evm_rpc_types
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize, Clone)]".into());
    builder.add(evm_rpc_types);

    builder.build(Some(manifest_dir.join("src/evm_rpc")));
}
