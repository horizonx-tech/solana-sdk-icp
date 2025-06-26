use anyhow::Result;
use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::main::CanisterId;
use solana_program::pubkey::Pubkey;

use crate::signature::Signature;

use super::{signers::Signers, Signer, SignerError};

#[derive(Clone, Debug)]
pub struct ThresholdSigner {
    key_id: SchnorrKeyIds,
    public_key: Pubkey,
}

#[derive(CandidType, Serialize, Debug)]
struct ManagementCanisterSchnorrPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ManagementCanisterSchnorrPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Clone)]
struct SchnorrKeyId {
    pub algorithm: SchnorrAlgorithm,
    pub name: String,
}
#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone)]
enum SchnorrAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    Bip340Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(CandidType, Serialize, Debug)]
struct ManagementCanisterSignatureRequest {
    pub message: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ManagementCanisterSignatureReply {
    pub signature: Vec<u8>,
}
impl ThresholdSigner {
    pub async fn new(key_id: SchnorrKeyIds) -> Result<Self> {
        let public_key = get_public_key(key_id).await?;
        Ok(Self::new_with_public_key(key_id, public_key))
    }
    pub fn new_with_public_key(key_id: SchnorrKeyIds, public_key: Pubkey) -> Self {
        Self { key_id, public_key }
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SchnorrKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl SchnorrKeyIds {
    fn to_key_id(&self, algorithm: SchnorrAlgorithm) -> SchnorrKeyId {
        SchnorrKeyId {
            algorithm,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}

impl ThresholdSigner {
    async fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        let result = sign(message, self.key_id)
            .await
            .map_err(|o| SignerError::Custom(o.to_string()))?;
        Ok(Signature::new(&result))
    }
}
#[async_trait]
impl Signer for ThresholdSigner {
    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        Ok(self.public_key)
    }
    async fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        self.try_sign_message(message).await
    }
    fn is_interactive(&self) -> bool {
        false
    }
}

pub async fn get_public_key(key_id: SchnorrKeyIds) -> Result<Pubkey> {
    let request = ManagementCanisterSchnorrPublicKeyRequest {
        canister_id: None,
        derivation_path: vec![ic_cdk::api::id().as_slice().to_vec()],
        key_id: key_id.to_key_id(SchnorrAlgorithm::Ed25519),
    };

    let (res,): (ManagementCanisterSchnorrPublicKeyReply,) = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (request,),
    )
    .await
    .map_err(|o| anyhow::anyhow!("Error calling schnorr_public_key: {:?}", o))?;
    let raw_public_key: Vec<u8> = res.public_key;
    Ok(Pubkey::new(&raw_public_key))
}
async fn sign(message: &[u8], key_ids: SchnorrKeyIds) -> Result<Vec<u8>> {
    let internal_request = ManagementCanisterSignatureRequest {
        message: message.to_vec(),
        derivation_path: vec![ic_cdk::api::id().as_slice().to_vec()],
        key_id: key_ids.to_key_id(SchnorrAlgorithm::Ed25519),
    };

    let (internal_reply,): (ManagementCanisterSignatureReply,) =
        ic_cdk::api::call::call_with_payment(
            Principal::management_canister(),
            "sign_with_schnorr",
            (internal_request,),
            35_000_000_000,
        )
        .await
        .map_err(|o| anyhow::anyhow!("Error calling sign_with_schnorr: {:?}", o))?;

    Ok(internal_reply.signature)
}
