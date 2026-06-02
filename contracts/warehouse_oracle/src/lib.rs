#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Admin,
    OraclePubkey,
    InspectorSet,
}

#[derive(Clone)]
#[contracttype]
pub struct InspectorSet {
    pub inspectors: Vec<Address>,
    pub threshold: u32,
}

#[contract]
pub struct WarehouseOracle;

#[contractimpl]
impl WarehouseOracle {
    pub fn initialize(env: Env, admin: Address, oracle_pubkey: Address, inspectors: InspectorSet) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::OraclePubkey, &oracle_pubkey);
        env.storage()
            .instance()
            .set(&DataKey::InspectorSet, &inspectors);
    }
}
