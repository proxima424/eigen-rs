#![doc(
    html_logo_url = "https://github.com/supernovahs/eigensdk-rs/assets/91280922/bd13caec-3c00-4afc-839a-b83d2890beb5",
    issue_tracker_base_url = "https://github.com/supernovahs/eigen-rs/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod binding;
use alloy_network::{Ethereum, EthereumSigner};
use alloy_provider::{
    fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, SignerFiller},
    ProviderBuilder, RootProvider,
};
use alloy_signer_wallet::LocalWallet;
use alloy_transport_http::{Client, Http};
use reqwest::Url;
use std::fs;
use std::str::FromStr;
pub fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn get_signer(
    key: String,
    rpc_url: &String,
) -> FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<alloy_provider::Identity, GasFiller>, NonceFiller>,
            ChainIdFiller,
        >,
        SignerFiller<EthereumSigner>,
    >,
    RootProvider<alloy_transport_http::Http<Client>>,
    alloy_transport_http::Http<Client>,
    Ethereum,
> {
    let wallet = LocalWallet::from_str(&key.to_string()).expect("failed to generate wallet ");
    let url = Url::parse(&rpc_url).expect("Wrong rpc url");
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(wallet.clone()))
        .on_http(url);

    return provider;
}

pub fn get_provider(
    rpc_url: &String,
) -> FillProvider<
    JoinFill<JoinFill<JoinFill<alloy_provider::Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
> {
    let url = Url::parse(&rpc_url).expect("Wrong rpc url");
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(url);

    provider
}
