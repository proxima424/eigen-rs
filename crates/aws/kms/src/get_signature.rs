use aws_config::meta::region::RegionProviderChain;
use aws_sdk_kms::Client;
use aws_sdk_kms::types::{MessageType, SigningAlgorithmSpec};
use asn1::SimpleAsn1Readable;
use std::error::Error;

#[derive(Debug, asn1::SimpleAsn1Readable)]
struct Asn1EcSig<'a> {
    r: asn1::RawValue<'a>,
    s: asn1::RawValue<'a>,
}

async fn get_ecdsa_signature(
    client: &Client,
    key_id: &str,
    msg: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let sign_input = aws_sdk_kms::input::SignInput::builder()
        .key_id(key_id)
        .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
        .message_type(MessageType::Digest)
        .message(msg)
        .build();

    let sign_output = client.sign(sign_input).await?;

    let asn1_sig: Asn1EcSig = asn1::from_der(&sign_output.signature)?;
    let r = asn1_sig.r.as_bytes().to_vec();
    let s = asn1_sig.s.as_bytes().to_vec();

    Ok((r, s))
}

