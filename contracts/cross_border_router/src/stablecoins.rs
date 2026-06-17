use soroban_sdk::{Address, Env, Symbol};

use crate::DataKey;

pub const SYMBOL_USDC: &str = "USDC";
pub const SYMBOL_CNGN: &str = "cNGN";
pub const SYMBOL_CXOF: &str = "cXOF";
pub const SYMBOL_CGHS: &str = "cGHS";
pub const SYMBOL_CKES: &str = "cKES";

pub fn register_asset(env: &Env, symbol: &Symbol, contract_id: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::Asset(symbol.clone()), contract_id);
    env.events().publish(
        (Symbol::new(env, "AssetRegistered"), symbol.clone()),
        contract_id.clone(),
    );
}

pub fn get_asset(env: &Env, symbol: &Symbol) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Asset(symbol.clone()))
        .expect("Asset not registered")
}
