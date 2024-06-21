use aws_config::meta::region::RegionProviderChain;
use aws_sdk_kms::Client;
use aws_sdk_kms::types::GetPublicKeyInput;
use std::error::Error;
use asn1::SimpleAsn1Readable;
use k256::ecdsa::VerifyingKey;
use k256::pkcs8::DecodePublicKey;

#[derive(Debug, SimpleAsn1Readable)]
struct Asn1EcPublicKey<'a> {
    ec_public_key_info: Asn1EcPublicKeyInfo<'a>,
    public_key: asn1::BitString<'a>,
}

#[derive(Debug, SimpleAsn1Readable)]
struct Asn1EcPublicKeyInfo<'a> {
    algorithm: asn1::ObjectIdentifier<'a>,
    parameters: asn1::ObjectIdentifier<'a>,
}

async fn GetECDSAPublicKey (client: &Client, key_id: &str) -> Result<VerifyingKey, Box<dyn Error>> {
    let get_pub_key_output = client
        .get_public_key()
        .key_id(key_id)
        .send()
        .await?;

    let asn1_pubk: Asn1EcPublicKey = asn1::from_der(&get_pub_key_output.public_key)?;
    let pubkey = VerifyingKey::from_public_key_der(asn1_pubk.public_key.as_bytes())?;

    Ok(pubkey)
}
