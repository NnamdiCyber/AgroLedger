#![no_std]
use soroban_sdk::{Bytes, contract, contractimpl, contracttype, Address, Env, Symbol};

#[contracttype]
pub enum DataKey {
    Admin,
    WarehouseOracle,
    ComplianceRegistry,
    Lot(u64),
    LotCounter,
}

#[derive(Clone)]
#[contracttype]
pub struct LotMeta {
    pub warehouse_id: Address,
    pub commodity: Symbol,
    pub quantity_kg: u64,
    pub oracle_attestation: Bytes,
    pub expiry: u64,
    pub price: i128,
}

#[contract]
pub struct CropToken;

#[contractimpl]
impl CropToken {
    pub fn initialize(
        env: Env,
        admin: Address,
        warehouse_oracle: Address,
        compliance_registry: Address,
    ) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::WarehouseOracle, &warehouse_oracle);
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRegistry, &compliance_registry);
        env.storage().instance().set(&DataKey::LotCounter, &0u64);
    }
}
