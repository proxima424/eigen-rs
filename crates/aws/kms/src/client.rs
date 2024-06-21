use aws_config::meta::region::RegionProviderChain;
use aws_sdk_kms::Client;
use std::error::Error;

async fn NewKMSClient(region : &str) -> Result<Client, Box<dyn Error>> {
    let region_provider = RegionProviderChain::default_provider().or_else(region);
    let config = aws_config::from_env().region(region_provider).load().await?;
    let client = Client::new(&config);
    Ok(Client);
}



