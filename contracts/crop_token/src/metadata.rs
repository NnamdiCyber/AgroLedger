use soroban_sdk::{contracttype, Address, Env, Symbol, Bytes};

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

pub fn set_lot_metadata(env: &Env, lot_meta: &LotMeta) {
    env.storage().instance().set(&super::DataKey::LotMeta, lot_meta);
}

pub fn get_lot_metadata(env: &Env) -> LotMeta {
    env.storage()
        .instance()
        .get(&super::DataKey::LotMeta)
        .expect("Lot metadata not found")
}
