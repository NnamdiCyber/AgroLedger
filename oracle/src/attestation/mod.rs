use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotAttestationRequest {
    pub warehouse_id: String,
    pub lot_id: String,
    pub commodity: String,
    pub quantity_kg: u64,
    pub inspector_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotAttestation {
    pub warehouse_id: String,
    pub lot_id: String,
    pub commodity: String,
    pub quantity_kg: u64,
    pub signature: String,
    pub signer: String,
    pub signed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResponse {
    pub success: bool,
    pub attestation: Option<LotAttestation>,
    pub error: Option<String>,
}

fn hash_lot_payload(req: &LotAttestationRequest) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(req.warehouse_id.as_bytes());
    hasher.update(b"|");
    hasher.update(req.lot_id.as_bytes());
    hasher.update(b"|");
    hasher.update(req.commodity.as_bytes());
    hasher.update(b"|");
    hasher.update(req.quantity_kg.to_string().as_bytes());
    hasher.finalize().to_vec()
}

fn create_attestation(
    req: &LotAttestationRequest,
    secret_key: &str,
) -> LotAttestation {
    let payload_hash = hash_lot_payload(req);
    let signature = hex::encode(payload_hash);

    LotAttestation {
        warehouse_id: req.warehouse_id.clone(),
        lot_id: req.lot_id.clone(),
        commodity: req.commodity.clone(),
        quantity_kg: req.quantity_kg,
        signature,
        signer: secret_key.chars().take(8).collect(),
        signed_at: Utc::now().timestamp() as u64,
    }
}

pub fn sign_lot(req: &LotAttestationRequest, config: &crate::OracleConfig) -> AttestationResponse {
    if req.quantity_kg > config.attestation.max_lot_weight_kg {
        return AttestationResponse {
            success: false,
            attestation: None,
            error: Some(format!(
                "Lot exceeds max weight of {} kg",
                config.attestation.max_lot_weight_kg
            )),
        };
    }

    let attestation = create_attestation(req, &config.signing.secret_key);

    AttestationResponse {
        success: true,
        attestation: Some(attestation),
        error: None,
    }
}

pub async fn run_attestation_server(_state: Arc<AppState>) {
    println!("[attestation] Stub server started (port not bound in skeleton)");
    println!("[attestation] Use sign_lot() directly for offline signing");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
