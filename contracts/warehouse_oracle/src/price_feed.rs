use soroban_sdk::{Address, Env, Symbol};
use crate::{DataKey, PriceData};

pub fn execute_push_price(env: Env, commodity: Symbol, price_usdc: u64, timestamp: u64) {
    let oracle: Address = env.storage().instance().get(&DataKey::OraclePubkey).unwrap();
    oracle.require_auth();

    let price_data = PriceData {
        price: price_usdc,
        timestamp,
    };
    env.storage()
        .instance()
        .set(&DataKey::Price(commodity.clone()), &price_data);

    env.events().publish(
        (Symbol::new(&env, "PriceUpdated"), commodity),
        (price_usdc, timestamp),
    );
}

pub fn execute_get_price(env: Env, commodity: Symbol) -> PriceData {
    env.storage()
        .instance()
        .get(&DataKey::Price(commodity))
        .unwrap_or(PriceData {
            price: 0,
            timestamp: 0,
        })
}
