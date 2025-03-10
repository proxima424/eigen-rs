//! get operators stake in quorums at current block
use alloy_primitives::FixedBytes;
use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_testing_utils::m2_holesky_constants::{
    BLS_APK_REGISTRY, OPERATOR_STATE_RETRIEVER, REGISTRY_COORDINATOR, STAKE_REGISTRY,
};
use eyre::Result;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let holesky_provider = "https://holesky.drpc.org";
    let avs_registry = AvsRegistryChainReader::new(
        REGISTRY_COORDINATOR,
        BLS_APK_REGISTRY,
        OPERATOR_STATE_RETRIEVER,
        STAKE_REGISTRY,
        holesky_provider.to_string(),
    );
    let operators_state = avs_registry
        .get_operator_stake_in_quorums_of_operator_at_current_block(
            FixedBytes::from_str(
                "0x9bdde6f82077712c6e1c9aa8e7fca47529effb948faafa1fa21aebd343fc7fec",
            )
            .expect("wrong operator id"),
        )
        .await
        .unwrap();

    println!("operator state at current block is {:?}", operators_state);

    Ok(())
}
