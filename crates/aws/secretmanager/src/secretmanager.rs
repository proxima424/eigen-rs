use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::Client;
use aws_sdk_secretsmanager::types::GetSecretValueInput;
use std::error::Error;

async fn ReadStringFromSecretManager(secret_name : &str, region : &str) -> Result< String, Box<>dyn Error> {
    let region_provider = RegionProviderChain::default_provider().or_else(region);
    let config = aws_config::from_env().region(region_provider).load().await?;

    let client = Client::new(&config);

    let input = GetSecretValueInput::builder()
        .secret_id(secret_name)
        .version_stage("AWSCURRENT")
        .build();

    let result = client.get_secret_value(input).await?;
    let secret_string = result.secret_string().unwrap_or_default();

    Ok(secret_string);

}