use crate::error::AvsRegistryError;
use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_provider::Provider;
use alloy_rpc_types::Filter;
use ark_ff::Zero;
use eigen_types::operator::{bitmap_to_quorum_ids, OperatorPubKeys};
use eigen_utils::{
    binding::{BLSApkRegistry, OperatorStateRetriever, RegistryCoordinator, StakeRegistry},
    get_provider,
};
use num_bigint::BigInt;
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::debug;

/// Avs Registry chainreader
#[derive(Debug, Clone)]
pub struct AvsRegistryChainReader {
    bls_apk_registry_addr: Address,
    registry_coordinator_addr: Address,
    operator_state_retriever: Address,
    stake_registry_addr: Address,
    provider: String,
}

trait AvsRegistryReader {
    fn get_quorum_count() -> Result<u8, String>;
}

impl AvsRegistryChainReader {
    /// New AvsRegistryChainReader instance
    pub fn new(
        registry_coordinator_addr: Address,
        bls_apk_registry_addr: Address,
        operator_state_retriever: Address,
        stake_registry_addr: Address,
        provider: String,
    ) -> Self {
        AvsRegistryChainReader {
            bls_apk_registry_addr,
            registry_coordinator_addr,
            operator_state_retriever,
            stake_registry_addr,
            provider,
        }
    }

    pub async fn build_avs_registry_chain_reader(
        &self,
        registry_coordinator_addr: Address,
        operator_state_retriever_addr: Address,
        stake_registry_addr: Address,
    ) -> Result<AvsRegistryChainReader, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, &provider);

        let bls_apk_registry_addr = contract_registry_coordinator.blsApkRegistry().call().await;

        match bls_apk_registry_addr {
            Ok(address) => {
                let RegistryCoordinator::blsApkRegistryReturn { _0: addres } = address;

                Ok(AvsRegistryChainReader {
                    bls_apk_registry_addr: addres,
                    registry_coordinator_addr,
                    operator_state_retriever: operator_state_retriever_addr,
                    stake_registry_addr,
                    provider: self.provider.clone(),
                })
            }

            Err(_) => Err(Box::new(AvsRegistryError::GetBlsApkRegistry)),
        }
    }

    pub async fn get_quorum_count(&self) -> Result<u8, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let quorum_count_result = contract_registry_coordinator.quorumCount().call().await;

        match quorum_count_result {
            Ok(quorum_count) => {
                let RegistryCoordinator::quorumCountReturn { _0: quorum } = quorum_count;
                return Ok(quorum);
            }

            Err(_) => Err(Box::new(AvsRegistryError::GetQuorumCount)),
        }
    }

    /// Get operators stake in quorums at a particular block
    pub async fn get_operators_stake_in_quorums_at_block(
        &self,
        block_number: u32,
        quorum_numbers: Bytes,
    ) -> Result<Vec<Vec<OperatorStateRetriever::Operator>>, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_operator_state_retriever =
            OperatorStateRetriever::new(self.operator_state_retriever, provider);
        let operator_state_result = contract_operator_state_retriever
            .getOperatorState_0(self.registry_coordinator_addr, quorum_numbers, block_number)
            .call()
            .await;

        match operator_state_result {
            Ok(operator_state) => {
                let OperatorStateRetriever::getOperatorState_0Return { _0: quorum } =
                    operator_state;
                return Ok(quorum);
            }
            Err(_) => {
                return Err(Box::new(AvsRegistryError::GetOperatorState));
            }
        }
    }

    pub async fn get_operators_stake_in_quorums_at_block_operator_id(
        &self,
        block_number: u32,
        operator_id: B256,
    ) -> Result<(U256, Vec<Vec<OperatorStateRetriever::Operator>>), Box<dyn std::error::Error>>
    {
        let provider = get_provider(&self.provider);

        let contract_operator_state_retriever =
            OperatorStateRetriever::new(self.operator_state_retriever, provider);
        let operator_state_with_registry_coordinator_and_oeprator_id_result =
            contract_operator_state_retriever
                .getOperatorState_1(
                    self.registry_coordinator_addr,
                    operator_id.into(),
                    block_number,
                )
                .call()
                .await;

        match operator_state_with_registry_coordinator_and_oeprator_id_result {
            Ok(operator_state_with_registry_coordinator_and_oeprator_id) => {
                let OperatorStateRetriever::getOperatorState_1Return {
                    _0: stake,
                    _1: operator_state,
                } = operator_state_with_registry_coordinator_and_oeprator_id;
                return Ok((stake, operator_state));
            }
            Err(_) => {
                return Err(Box::new(
                    AvsRegistryError::GetOperatorStateWithRegistryCoordinatorAndOperatorId,
                ));
            }
        }
    }

    pub async fn get_operators_stake_in_quorums_at_current_block(
        &self,
        quorum_numbers: Bytes,
    ) -> Result<Vec<Vec<OperatorStateRetriever::Operator>>, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let current_block_number_result = provider.get_block_number().await;

        match current_block_number_result {
            Ok(current_block_number) => {
                if current_block_number > u32::MAX.into() {
                    return Err(Box::new(AvsRegistryError::BlockNumberOverflow));
                }

                let operators_stake_in_quorums_at_block_result = self
                    .get_operators_stake_in_quorums_at_block(
                        current_block_number as u32,
                        quorum_numbers,
                    )
                    .await;

                match operators_stake_in_quorums_at_block_result {
                    Ok(operators_stake_in_quorums_at_block) => {
                        Ok(operators_stake_in_quorums_at_block)
                    }
                    Err(_) => {
                        return Err(Box::new(
                            AvsRegistryError::GetOperatorStakeInQuorumAtBlockNumber,
                        ))
                    }
                }
            }
            Err(_) => return Err(Box::new(AvsRegistryError::GetBlockNumber)),
        }
    }

    pub async fn get_operators_stake_in_quorums_of_operator_at_block(
        &self,
        operator_id: B256,
        block_number: u32,
    ) -> Result<(Vec<u8>, Vec<Vec<OperatorStateRetriever::Operator>>), Box<dyn (std::error::Error)>>
    {
        let result_ = self
            .get_operators_stake_in_quorums_at_block_operator_id(block_number, operator_id)
            .await;

        match result_ {
            Ok((quorum_bitmaps, operator_stakes)) => {
                let quorums = bitmap_to_quorum_ids(quorum_bitmaps);

                let s = (quorums, operator_stakes);
                return Ok(s);
            }
            Err(_) => {
                return Err(Box::new(
                    AvsRegistryError::GetOperatorStakeInQuorumAtBlockOperatorId,
                ))
            }
        }
    }

    pub async fn get_operators_stake_in_quorums_of_operator_at_current_block(
        &self,
        operator_id: B256,
    ) -> Result<(Vec<u8>, Vec<Vec<OperatorStateRetriever::Operator>>), Box<dyn std::error::Error>>
    {
        let provider = get_provider(&self.provider);

        let current_block_number = provider.get_block_number().await?;

        if current_block_number > u32::MAX.into() {
            return Err(Box::new(AvsRegistryError::BlockNumberOverflow));
        }

        let operator_stake_in_quorum_of_operaotr_at_block = self
            .get_operators_stake_in_quorums_of_operator_at_block(
                operator_id,
                current_block_number as u32,
            )
            .await?;
        Ok(operator_stake_in_quorum_of_operaotr_at_block)
    }

    pub async fn get_operator_stake_in_quorums_of_operator_at_current_block(
        &self,
        operator_id: B256,
    ) -> Result<HashMap<u8, BigInt>, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, &provider);

        let quorum_bitmap = registry_coordinator
            .getCurrentQuorumBitmap(operator_id.into())
            .call()
            .await?;

        let RegistryCoordinator::getCurrentQuorumBitmapReturn { _0: quo } = quorum_bitmap;

        let quorums = bitmap_to_quorum_ids(quo);

        let mut quorum_stakes: HashMap<u8, BigInt> = HashMap::new();

        let stake_registry = StakeRegistry::new(self.stake_registry_addr, &provider);
        for quorum in quorums.iter() {
            let stakes_result = stake_registry
                .getCurrentStake(operator_id.into(), *quorum)
                .call()
                .await?;

            let StakeRegistry::getCurrentStakeReturn { _0: c_stake } = stakes_result;
            quorum_stakes.insert(*quorum, c_stake.into());
        }
        Ok(quorum_stakes)
    }

    ///
    pub async fn get_check_signatures_indices(
        &self,
        reference_block_number: u32,
        quorum_numbers: Vec<u8>,
        non_signer_operator_ids: Vec<FixedBytes<32>>,
    ) -> Result<OperatorStateRetriever::CheckSignaturesIndices, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_operator_state_retriever =
            OperatorStateRetriever::new(self.operator_state_retriever, provider);

        let check_signature_indices = contract_operator_state_retriever
            .getCheckSignaturesIndices(
                self.registry_coordinator_addr,
                reference_block_number,
                quorum_numbers.into(),
                non_signer_operator_ids,
            )
            .call()
            .await?;
        let OperatorStateRetriever::getCheckSignaturesIndicesReturn { _0: indices } =
            check_signature_indices;
        Ok(indices)
    }

    pub async fn get_operator_id(
        &self,
        operator_address: Address,
    ) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let operator_id_return = contract_registry_coordinator
            .getOperatorId(operator_address)
            .call()
            .await?;
        let RegistryCoordinator::getOperatorIdReturn { _0: operator_id } = operator_id_return;
        Ok(operator_id)
    }

    /// Get Operator from operator id
    pub async fn get_operator_from_id(
        &self,
        operator_id: [u8; 32],
    ) -> Result<Address, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, &provider);

        let operator_address_return = contract_registry_coordinator
            .getOperatorFromId(operator_id.into())
            .call()
            .await?;

        let RegistryCoordinator::getOperatorFromIdReturn {
            _0: operator_address,
        } = operator_address_return;
        Ok(operator_address)
    }

    pub async fn is_operator_registered(
        &self,
        operator_address: Address,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let operator_status_return = contract_registry_coordinator
            .getOperatorStatus(operator_address)
            .call()
            .await?;

        let RegistryCoordinator::getOperatorStatusReturn {
            _0: operator_status,
        } = operator_status_return;

        Ok(operator_status == 1)
    }

    /// Queies existing operators from for a particular block range
    pub async fn query_existing_registered_operator_pub_keys(
        &self,
        start_block: u64,
        mut stop_block: u64,
    ) -> Result<(Vec<Address>, Vec<OperatorPubKeys>), Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let query_block_range = 1024;
        let current_block_number = provider.get_block_number().await?;
        if stop_block.is_zero() {
            stop_block = current_block_number;
        }
        let mut i = start_block;
        let mut operator_addresses: Vec<Address> = vec![];
        let mut operator_pub_keys: Vec<OperatorPubKeys> = vec![];
        while i <= stop_block {
            let mut to_block = i + (query_block_range - 1);
            if to_block > stop_block {
                to_block = stop_block;
            }
            let filter = Filter::new()
                .select(i..to_block)
                .event("NewPubkeyRegistration(address,(uint256,uint256),(uint256[2],uint256[2]))")
                .address(self.bls_apk_registry_addr);

            let logs = provider.get_logs(&filter).await?;
            debug!(transactionLogs = ?logs, "avsRegistryChainReader.QueryExistingRegisteredOperatorPubKeys");

            for v_log in logs.iter() {
                let pub_key_reg_option = v_log
                    .log_decode::<BLSApkRegistry::NewPubkeyRegistration>()
                    .ok();
                if let Some(pub_key_reg) = pub_key_reg_option {
                    let data = pub_key_reg.data();
                    let operator_addr = data.operator;
                    operator_addresses.push(operator_addr);
                    let g1_pub_key = data.pubkeyG1.clone();
                    let g2_pub_key = data.pubkeyG2.clone();

                    let operator_pub_key = OperatorPubKeys {
                        g1_pub_key: g1_pub_key,
                        g2_pub_key: g2_pub_key,
                    };

                    operator_pub_keys.push(operator_pub_key);
                }
            }
            i += 1024;
        }
        Ok((operator_addresses, operator_pub_keys))
    }

    /// Query existing operator sockets
    pub async fn query_existing_registered_operator_sockets(
        &self,
        start_block: u64,
        stop_block: u64,
    ) -> Result<HashMap<FixedBytes<32>, String>, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let mut operator_id_to_socket = HashMap::new();

        let query_block_range = 10000;

        let mut i = start_block;

        while i <= stop_block {
            let mut to_block = i + (query_block_range - 1);
            if to_block > stop_block {
                to_block = stop_block;
            }

            let mut filter = Filter::new()
                .select(start_block..to_block)
                .event("OperatorSocketUpdate(bytes32,string)")
                .address(self.registry_coordinator_addr);
            if stop_block == 0 {
                let current_block_number = provider.get_block_number().await?;

                filter = filter.clone().select(start_block..current_block_number);
            };

            let logs = provider.get_logs(&filter).await?;

            for v_log in logs.iter() {
                let socket_update_filter_option = v_log
                    .log_decode::<RegistryCoordinator::OperatorSocketUpdate>()
                    .ok();
                if let Some(socket_update_filter) = socket_update_filter_option {
                    let data = socket_update_filter.data();
                    let operator_id = data.operatorId;
                    let socket = &data.socket;
                    operator_id_to_socket.insert(operator_id, socket.clone());
                }
            }

            i += query_block_range;
        }

        Ok(operator_id_to_socket)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_primitives::keccak256;
    use hex::FromHex;
    use std::str::FromStr;
    const HOLESKY_REGISTRY_COORDINATOR: &str = "0x53012C69A189cfA2D9d29eb6F19B32e0A2EA3490";
    const HOLESKY_OPERATOR_STATE_RETRIEVER: &str = "0xB4baAfee917fb4449f5ec64804217bccE9f46C67";
    const HOLESKY_STAKE_REGISTRY: &str = "0xBDACD5998989Eec814ac7A0f0f6596088AA2a270";
    const HOLESKY_BLS_APK_REGISTRY: &str = "0x066cF95c1bf0927124DFB8B02B401bc23A79730D";

    #[tokio::test]
    async fn test_build_avs_registry_chain_reader() {
        let provider = "http://localhost:8545";
        let instance = AvsRegistryChainReader::new(
            Address::from_word(keccak256("registry")),
            // Address::from_word(keccak256(("registry").into())),
            Address::from_word(keccak256("blsapkregistry")),
            Address::from_word(keccak256("operatorstateretriever")),
            Address::from_word(keccak256("stakeregistry")),
            provider.to_string(),
        );
        let _ = AvsRegistryChainReader::build_avs_registry_chain_reader(
            &instance,
            Address::from_word(keccak256("registry")),
            Address::from_word(keccak256("operator")),
            Address::from_word(keccak256("stake")),
        );
    }
    fn build_avs_registry_chain_reader() -> AvsRegistryChainReader {
        let holesky_registry_coordinator =
            Address::from_str(HOLESKY_REGISTRY_COORDINATOR).expect("failed to parse address");
        let holesky_operator_state_retriever =
            Address::from_str(HOLESKY_OPERATOR_STATE_RETRIEVER).expect("failed to parse address");

        let holesky_stake_registry =
            Address::from_str(HOLESKY_STAKE_REGISTRY).expect("failed to parse address");

        let holesky_bls_apk_registry =
            Address::from_str(HOLESKY_BLS_APK_REGISTRY).expect("failed to parse address");

        let holesky_provider = "https://ethereum-holesky.blockpi.network/v1/rpc/public";
        let avs_registry_chain_reader = AvsRegistryChainReader::new(
            holesky_registry_coordinator,
            holesky_bls_apk_registry,
            holesky_operator_state_retriever,
            holesky_stake_registry,
            holesky_provider.to_string(),
        );

        return avs_registry_chain_reader;
    }

    #[tokio::test]
    async fn test_get_quorum_count() {
        let avs_reader = build_avs_registry_chain_reader();

        let _ = avs_reader.get_quorum_count().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_at_block() {
        let avs_reader = build_avs_registry_chain_reader();

        let quorum_number = Bytes::from_hex("0x00").expect("bytes parse");
        let _ = avs_reader
            .get_operators_stake_in_quorums_at_block(1245063, quorum_number)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_at_block_operator_id() {
        let avs_reader = build_avs_registry_chain_reader();

        let operator_id = U256::from_str(
            "35344093966194310405039483339636912150346494903629410125452342281826147822033",
        )
        .unwrap();

        let _ = avs_reader
            .get_operators_stake_in_quorums_at_block_operator_id(1245842, operator_id.into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_at_current_block() {
        let avs_reader = build_avs_registry_chain_reader();
        let quorum_number = Bytes::from_hex("0x00").expect("bytes parse");

        let _ = avs_reader
            .get_operators_stake_in_quorums_at_current_block(quorum_number)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_of_operator_at_block() {
        let avs_reader = build_avs_registry_chain_reader();

        let operator_id = U256::from_str(
            "35344093966194310405039483339636912150346494903629410125452342281826147822033",
        )
        .unwrap();

        let _ = avs_reader
            .get_operators_stake_in_quorums_of_operator_at_block((operator_id).into(), 1246078)
            .await
            .unwrap();
    }
}
