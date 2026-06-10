use soroban_sdk::{contracttype, Env, Symbol, Bytes};

#[derive(Clone)]
#[contracttype]
pub struct LotMeta {
    pub warehouse_id: Symbol,
    pub lot_id: Symbol,
    pub commodity: Symbol,
    pub quantity_kg: u64,
    pub oracle_attestation: Bytes,
    pub expiry: u64,
    pub price: i128,
}

pub fn set_lot_metadata(env: &Env, lot_id: &Symbol, lot_meta: &LotMeta) {
    env.storage().instance().set(&super::DataKey::LotMeta(lot_id.clone()), lot_meta);
}

pub fn get_lot_metadata(env: &Env, lot_id: &Symbol) -> LotMeta {
    env.storage()
        .instance()
        .get(&super::DataKey::LotMeta(lot_id.clone()))
        .expect("Lot metadata not found")
}
